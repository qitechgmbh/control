use crate::aquapath1::{Flow, Temperature};
use control_core::controllers::pid::PidController;
use qitech_lib::ethercat_hal::io::analog_output::{AnalogOutputDevice, AnalogOutputOutput};
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::ethercat_hal::io::encoder_input::EncoderInputDevice;
use qitech_lib::ethercat_hal::io::temperature_input::TemperatureInputDevice;
use qitech_lib::units::VolumeRate;
use qitech_lib::{units};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use units::AngularVelocity;
use units::angular_velocity::revolution_per_minute;
use units::f64::ThermodynamicTemperature;
use units::thermodynamic_temperature::{degree_celsius, kelvin};
use units::volume_rate::liter_per_minute;

pub struct Controller {
    pub pid: PidController,
    window_start: Instant,
    pub temperature: Temperature,
    
    pub target_temperature: ThermodynamicTemperature,
    pub current_temperature: ThermodynamicTemperature,
    pub temp_reservoir: ThermodynamicTemperature,
    pub min_temperature: ThermodynamicTemperature,
    pub max_temperature: ThermodynamicTemperature,    
    pub cooling_tolerance: ThermodynamicTemperature,
    pub heating_tolerance: ThermodynamicTemperature,

    pub current_revolutions: AngularVelocity,
    pub max_revolutions: AngularVelocity,

    pub cooling_controller: Rc<RefCell<dyn AnalogOutputDevice>>,
    pub relais_control: Rc<RefCell<dyn DigitalOutputDevice>>,
    pub temperature_sensor_in: Rc<RefCell<dyn TemperatureInputDevice>>,
    pub flow_sensor: Rc<RefCell<dyn EncoderInputDevice>>,

    pub cooling_controller_port : usize,
    pub cooling_relais_port : usize,
    pub heating_relais_port : usize,
    pub temperature_port_in : usize,
    pub temperature_port_out : usize,
    pub pump_relais_port : usize,
    pub flow_sensor_port : usize,

    pub power: f64,
    pub total_energy: f64,

    pub cooling_allowed: bool,
    pub heating_allowed: bool,
    pub should_pump: bool,
    pub pump_allowed: bool,

    pub flow: Flow,
    pub current_flow: VolumeRate,
    pub max_flow: VolumeRate,


}

impl Controller {
    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        temp: Temperature,
        target_tempetature: ThermodynamicTemperature,
        cooling_controller: Rc<RefCell<dyn AnalogOutputDevice>>,
        relais_control: Rc<RefCell<dyn DigitalOutputDevice>>,
        temp_sensor_in: Rc<RefCell<dyn TemperatureInputDevice>>,
        max_revolutions: AngularVelocity,
        flow: Flow,
        flow_sensor: Rc<RefCell<dyn EncoderInputDevice>>,
        cooling_controller_port : usize,
        cooling_relais_port : usize,
        heating_relais_port : usize,
        temperature_port_in : usize,
        temperature_port_out : usize,
        pump_relais_port : usize,
        flow_sensor_port : usize,

    ) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd),
            window_start: Instant::now(),
            target_temperature: target_tempetature,
            current_temperature: ThermodynamicTemperature::new::<degree_celsius>(25.0),
            temp_reservoir: ThermodynamicTemperature::new::<degree_celsius>(25.0),
            min_temperature: ThermodynamicTemperature::new::<degree_celsius>(10.0),
            max_temperature: ThermodynamicTemperature::new::<degree_celsius>(50.0),
            temperature: temp,
            cooling_controller: cooling_controller,
            cooling_tolerance: ThermodynamicTemperature::new::<degree_celsius>(2.0),
            heating_tolerance: ThermodynamicTemperature::new::<degree_celsius>(2.0),
            current_revolutions: AngularVelocity::new::<revolution_per_minute>(0.0),
            max_revolutions: max_revolutions,
            cooling_allowed: false,
            heating_allowed: false,
            temperature_sensor_in: temp_sensor_in,
            power: 700.0,
            total_energy: 0.0,
            flow: flow,
            flow_sensor: flow_sensor,
            should_pump: false,
            current_flow: VolumeRate::new::<liter_per_minute>(0.0),
            pump_allowed: false,
            max_flow: VolumeRate::new::<liter_per_minute>(10.0),            
            cooling_controller_port,
            cooling_relais_port,
            heating_relais_port,
            temperature_port_in,
            temperature_port_out,
            pump_relais_port,
            flow_sensor_port,
            relais_control,            
        }
    }

    pub fn turn_pump_off(&mut self) {
        self.flow.pump = false;
        let relais = &mut *self.relais_control.borrow_mut();
        relais.set_output(self.pump_relais_port,false);
    }

    pub fn turn_pump_on(&mut self) {
        self.flow.pump = true;
        let relais = &mut *self.relais_control.borrow_mut();
        relais.set_output(self.pump_relais_port,true);
    }

    pub fn disallow_pump(&mut self) {
        self.pump_allowed = false;
    }

    pub fn allow_pump(&mut self) {
        self.pump_allowed = true;
    }

    pub fn get_pump(&mut self) -> bool {
        self.pump_allowed
    }

    pub fn disable_cooling(&mut self) {
        self.turn_cooling_off();
        self.disallow_cooling();
    }

    pub fn disable_heating(&mut self) {
        self.turn_heating_off();
        self.disallow_heating();
    }

    pub fn enable_cooling(&mut self) {
        self.turn_cooling_on();
        self.allow_cooling();
    }

    pub fn enable_heating(&mut self) {
        self.turn_heating_on();
        self.allow_heating();
    }
    pub fn reset_pid(&mut self) {
        self.pid.reset()
    }
    pub fn set_target_temperature(&mut self, temperature: ThermodynamicTemperature) {
        self.reset_pid();
        self.target_temperature = temperature;
    }

    pub fn get_temp_in(&mut self) -> ThermodynamicTemperature {
        let temp_sensor = self.temperature_sensor_in.borrow_mut();
        let temp = temp_sensor.get_input(self.temperature_port_in);
        match temp {
            Ok(value) => ThermodynamicTemperature::new::<degree_celsius>(value.temperature as f64),
            Err(_) => ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }

    pub fn get_temp_out(&mut self) -> ThermodynamicTemperature {
        let temp_sensor = self.temperature_sensor_in.borrow_mut();
        let temp = temp_sensor.get_input(self.temperature_port_out);
        match temp {
            Ok(value) => ThermodynamicTemperature::new::<degree_celsius>(value.temperature as f64),
            Err(_) => ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }

    pub fn disallow_cooling(&mut self) {
        self.cooling_allowed = false;
    }

    pub fn allow_cooling(&mut self) {
        self.cooling_allowed = true;
    }

    pub fn disallow_heating(&mut self) {
        self.heating_allowed = false;
    }

    pub fn allow_heating(&mut self) {
        self.heating_allowed = true;
    }

    pub fn turn_cooling_on(&mut self) {
        let relais = &mut *self.relais_control.borrow_mut();
        relais.set_output(self.cooling_relais_port, true);
        self.temperature.cooling = true;
    }

    pub fn turn_cooling_off(&mut self) {
        let relais = &mut *self.relais_control.borrow_mut();
        relais.set_output(self.cooling_relais_port, false);
        self.current_revolutions = AngularVelocity::new::<revolution_per_minute>(0.0);
        self.temperature.cooling = false;
    }

    pub fn turn_heating_on(&mut self) {
        let relais = &mut *self.relais_control.borrow_mut();
        relais.set_output(self.heating_relais_port, true);
        self.temperature.heating = true;
    }

    pub fn turn_heating_off(&mut self) {
        let relais = &mut *self.relais_control.borrow_mut();
        relais.set_output(self.heating_relais_port, false);
        self.temperature.heating = false;
        self.power = 0.0;
    }

    pub fn set_should_pump(&mut self, should_pump: bool) {
        self.should_pump = should_pump;
    }

    pub fn get_should_pump(&mut self) -> bool {
        self.should_pump
    }

    pub fn get_flow(&mut self) -> VolumeRate {
        let flow_sensor = self.flow_sensor.borrow();
        let freq_result = flow_sensor.get_frequency(self.flow_sensor_port);
        drop(flow_sensor);
        let value = match freq_result {
            Ok(val) => val,
            Err(_e) => {
                return VolumeRate::new::<liter_per_minute>(0.0);
            }
        };

        match value {
            Some(val) => {
                if val.value == 0 {
                    return VolumeRate::new::<liter_per_minute>(0.0);
                }
                // Formula: f = 8.1*q - 3, so q = (f + 3) / 8.1
                let actual_flow = ((val.value / 100) as f32 + 3.0) / 8.1;
                VolumeRate::new::<liter_per_minute>(actual_flow.into())
            }
            None => {
                return VolumeRate::new::<liter_per_minute>(0.0);
            }
        }
    }

    pub fn get_current_revolutions(&self) -> AngularVelocity {
        self.current_revolutions
    }

    pub fn get_max_revolutions(&self) -> AngularVelocity {
        self.max_revolutions
    }

    pub fn set_max_revolutions(&mut self, revolutions: AngularVelocity) {
        self.max_revolutions = revolutions;
    }

    pub fn set_cooling_tolerance(&mut self, tolerance: ThermodynamicTemperature) {
        self.cooling_tolerance = tolerance;
    }

    pub fn set_heating_tolerance(&mut self, tolerance: ThermodynamicTemperature) {
        self.heating_tolerance = tolerance;
    }

    // no power/energy unit implemented
    pub fn get_current_power(&self) -> f64 {
        self.power
    }

    pub fn get_total_energy(&self) -> f64 {
        self.total_energy
    }

    pub fn update(&mut self, now: Instant) -> () {
        let current_flow = self.get_flow();
        self.current_flow = current_flow;
        self.flow.flow = current_flow;

        let should_flow = self.get_should_pump();
        self.flow.should_pump = should_flow;

        if !self.flow.pump && self.get_pump() && should_flow {
            self.turn_pump_on();
        } else if self.flow.pump && (!self.get_pump() || !should_flow) {
            self.turn_pump_off();
        }

        self.current_temperature = self.get_temp_in();
        self.temp_reservoir = self.get_temp_out();

        if self.current_temperature < self.min_temperature && self.temperature.cooling {
            self.turn_cooling_off();
        } else if self.current_temperature > self.max_temperature && self.temperature.heating {
            self.turn_heating_off();
        }

        // Calculate PID error once
        let error = self.target_temperature.get::<degree_celsius>()
            - self.current_temperature.get::<degree_celsius>();

        let elapsed = now - self.window_start;
        self.window_start = now;

        // Decide whether to heat or cool based on error
        if error > self.heating_tolerance.get::<degree_celsius>() {
            // Need heating (current < target)
            if self.temperature.cooling {
                self.turn_cooling_off();
            }
            if self.heating_allowed && current_flow > VolumeRate::new::<liter_per_minute>(0.0) {
                self.turn_heating_on();

                self.total_energy += self.get_current_power() * elapsed.as_secs_f64() / 3600.0;
            } else {
                // Pump is off or heating not allowed - don't heat
                if self.temperature.heating {
                    self.turn_heating_off();
                }
            }
        } else if error < -self.cooling_tolerance.get::<degree_celsius>() {
            // Need cooling (current > target)
            if self.temperature.heating {
                self.turn_heating_off();
            }
            if self.cooling_allowed && current_flow > VolumeRate::new::<liter_per_minute>(0.0) {
                if !self.temperature.cooling {
                    self.turn_cooling_on();
                }

                let max_revolutions = self.get_max_revolutions();
                let temp_offset = self.current_temperature - self.target_temperature;

                let target_revolutions = (temp_offset.get::<kelvin>() * 10.0)
                    .clamp(0.0, max_revolutions.get::<revolution_per_minute>());
                
                let cooling_controller = &mut *self.cooling_controller.borrow_mut();
                let output = AnalogOutputOutput {0: target_revolutions as f32 / 10.0};
                cooling_controller.set_output(self.cooling_controller_port, output);
                self.current_revolutions =
                    AngularVelocity::new::<revolution_per_minute>(target_revolutions);
            } else {
                if self.temperature.cooling {
                    self.turn_cooling_off();
                }
            }
        }
    }
}
