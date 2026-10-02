use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use qitech_lib::ethercat_hal::io::analog_input::AnalogInputDevice;
use qitech_lib::ethercat_hal::io::analog_input::AnalogInputInput;
use qitech_lib::ethercat_hal::io::analog_input::physical::AnalogInputRange;
use qitech_lib::ethercat_hal::io::analog_output::AnalogOutputDevice;
use qitech_lib::ethercat_hal::io::as006::calculate_as006_flow_lpm;
use qitech_lib::ethercat_hal::io::as006::calculate_as006_temperature_celsius;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::units::ThermodynamicTemperature;
use qitech_lib::units::VolumeRate;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;
use qitech_lib::units::volume_rate::liter_per_minute;

/// One digital output port, remembering what it was last commanded to.
pub struct Relay {
    device: Rc<RefCell<dyn DigitalOutputDevice>>,
    port: usize,
    on: bool,
}

impl Relay {
    pub fn new(device: Rc<RefCell<dyn DigitalOutputDevice>>, port: usize) -> Self {
        Self {
            device,
            port,
            on: false,
        }
    }

    pub fn set(&mut self, on: bool) {
        self.device.borrow_mut().set_output(self.port, on);
        self.on = on;
    }

    pub fn is_on(&self) -> bool {
        self.on
    }
}

/// A relay that refuses to chatter: after switching it holds that state for a minimum
/// dwell time. Safety shutoffs bypass the dwell through [`DwellRelay::force_off`].
pub struct DwellRelay {
    relay: Relay,
    last_switch: Instant,
}

impl DwellRelay {
    pub fn new(device: Rc<RefCell<dyn DigitalOutputDevice>>, port: usize, now: Instant) -> Self {
        Self {
            relay: Relay::new(device, port),
            last_switch: now,
        }
    }

    pub fn is_on(&self) -> bool {
        self.relay.is_on()
    }

    /// Switches only if the state actually differs and the dwell has expired.
    /// Reports whether the switch happened.
    pub fn request(&mut self, on: bool, now: Instant, min_on: Duration, min_off: Duration) -> bool {
        if on == self.relay.is_on() {
            return false;
        }

        let dwell = if self.relay.is_on() { min_on } else { min_off };
        if now.duration_since(self.last_switch) < dwell {
            return false;
        }

        self.switch(on, now);
        true
    }

    /// Opens the relay immediately, ignoring the dwell. Reports whether it was on.
    pub fn force_off(&mut self, now: Instant) -> bool {
        if !self.relay.is_on() {
            return false;
        }

        self.switch(false, now);
        true
    }

    fn switch(&mut self, on: bool, now: Instant) {
        self.relay.set(on);
        self.last_switch = now;
    }
}

/// Analog fan-speed output.
pub struct FanOutput {
    device: Rc<RefCell<dyn AnalogOutputDevice>>,
    port: usize,
}

impl FanOutput {
    pub fn new(device: Rc<RefCell<dyn AnalogOutputDevice>>, port: usize) -> Self {
        Self { device, port }
    }

    /// The drive takes one tenth of the requested rpm as its analog setpoint.
    pub fn set_rpm(&mut self, rpm: f64) {
        self.device
            .borrow_mut()
            .set_output(self.port, (rpm as f32 / 10.0).into());
    }
}

/// One port of the AS006 combined flow/temperature probe.
///
/// A wiring fault or an out-of-range reading resolves to zero, matching how the
/// interlocks are written: no credible reading means no permission to heat.
struct As006Input {
    device: Rc<RefCell<dyn AnalogInputDevice>>,
    port: usize,
}

impl As006Input {
    fn new(device: Rc<RefCell<dyn AnalogInputDevice>>, port: usize) -> Self {
        Self { device, port }
    }

    fn read(&self, convert: impl Fn(&AnalogInputInput, &AnalogInputRange) -> Option<f64>) -> f64 {
        let device = self.device.borrow();
        let Ok(input) = device.get_input(self.port) else {
            return 0.0;
        };
        let range = device.analog_input_range();
        convert(&input, &range).unwrap_or(0.0)
    }
}

pub struct FlowSensor(As006Input);

impl FlowSensor {
    pub fn new(device: Rc<RefCell<dyn AnalogInputDevice>>, port: usize) -> Self {
        Self(As006Input::new(device, port))
    }

    pub fn read(&self) -> VolumeRate {
        VolumeRate::new::<liter_per_minute>(self.0.read(calculate_as006_flow_lpm))
    }
}

pub struct TemperatureSensor(As006Input);

impl TemperatureSensor {
    pub fn new(device: Rc<RefCell<dyn AnalogInputDevice>>, port: usize) -> Self {
        Self(As006Input::new(device, port))
    }

    pub fn read(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<degree_celsius>(
            self.0.read(calculate_as006_temperature_celsius),
        )
    }
}
