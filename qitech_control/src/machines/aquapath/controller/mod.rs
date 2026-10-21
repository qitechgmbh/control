mod config;
mod cooling;
mod heater;
mod io;
mod pump;
mod timing;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

pub use config::ControllerConfig;
pub use config::CoolingMode;
pub use config::PidGains;
use cooling::CoolingController;
use heater::HeaterController;
use io::DwellRelay;
use io::FanOutput;
use io::FlowSensor;
use io::Relay;
use io::TemperatureSensor;
use pump::PumpController;
use qitech_lib::ethercat_hal::io::analog_input::AnalogInputDevice;
use qitech_lib::ethercat_hal::io::analog_output::AnalogOutputDevice;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::ThermodynamicTemperature;
use qitech_lib::units::VolumeRate;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;

/// Why the accumulated control history was thrown away.
#[derive(Debug, Clone, Copy)]
pub enum ControlResetReason {
    TargetTemperatureChanged,
    HeatingToleranceChanged,
    CoolingToleranceChanged,
    PidParametersChanged,
    PumpCommandChanged,
}

/// Something the operator should be told about, drained once per act cycle.
#[derive(Debug, Clone, Copy)]
pub enum ControllerNotice {
    ControlReset(ControlResetReason),
    PumpStoppedLowFlow,
}

/// The devices and ports making up one side's half of the hardware.
///
/// Named fields rather than a long positional argument list: the ports are all `usize`
/// and swapping two of them silently drives the wrong relay.
pub struct ControllerHardware {
    pub relays: Rc<RefCell<dyn DigitalOutputDevice>>,
    pub fan: Rc<RefCell<dyn AnalogOutputDevice>>,
    pub sensor: Rc<RefCell<dyn AnalogInputDevice>>,
    pub pump_relay_port: usize,
    pub heating_relay_port: usize,
    pub cooling_relay_port: usize,
    pub fan_port: usize,
    pub flow_sensor_port: usize,
    pub temperature_sensor_port: usize,
}

/// What the loop wants this cycle, once the error is compared against the tolerances.
enum Demand {
    Heat,
    Cool,
    Hold,
}

/// One reservoir loop: a pump, a heating element and a fan, kept at a target temperature.
///
/// The safety ordering the whole thing exists to guarantee is that heat is never applied
/// without proven flow, and that flow always outlives the heat it has to carry away.
pub struct Controller {
    config: ControllerConfig,

    pump: PumpController,
    heater: HeaterController,
    cooling: CoolingController,
    temperature_sensor: TemperatureSensor,

    target_temperature: ThermodynamicTemperature,
    current_temperature: ThermodynamicTemperature,
    heating_tolerance: ThermodynamicTemperature,
    cooling_tolerance: ThermodynamicTemperature,
    heating_allowed: bool,

    last_update: Instant,
    notices: Vec<ControllerNotice>,
}

impl Controller {
    pub fn new(
        hardware: ControllerHardware,
        config: ControllerConfig,
        gains: PidGains,
        target_temperature: ThermodynamicTemperature,
        max_revolutions: AngularVelocity,
    ) -> Self {
        let now = Instant::now();

        let pump = PumpController::new(
            Relay::new(hardware.relays.clone(), hardware.pump_relay_port),
            FlowSensor::new(hardware.sensor.clone(), hardware.flow_sensor_port),
        );

        let heater = HeaterController::new(
            DwellRelay::new(hardware.relays.clone(), hardware.heating_relay_port, now),
            gains,
            now,
        );

        let cooling = CoolingController::new(
            DwellRelay::new(hardware.relays.clone(), hardware.cooling_relay_port, now),
            FanOutput::new(hardware.fan, hardware.fan_port),
            max_revolutions,
        );

        Self {
            pump,
            heater,
            cooling,
            temperature_sensor: TemperatureSensor::new(
                hardware.sensor,
                hardware.temperature_sensor_port,
            ),
            target_temperature,
            current_temperature: ThermodynamicTemperature::new::<degree_celsius>(25.0),
            heating_tolerance: config.heating_tolerance,
            cooling_tolerance: config.cooling.tolerance,
            heating_allowed: false,
            last_update: now,
            notices: Vec::new(),
            config,
        }
    }

    // --- readings and actuator state ---

    pub fn temperature(&self) -> ThermodynamicTemperature {
        self.current_temperature
    }

    pub fn target_temperature(&self) -> ThermodynamicTemperature {
        self.target_temperature
    }

    pub fn min_temperature(&self) -> ThermodynamicTemperature {
        self.config.min_temperature
    }

    pub fn max_temperature(&self) -> ThermodynamicTemperature {
        self.config.max_temperature
    }

    pub fn heating_tolerance(&self) -> ThermodynamicTemperature {
        self.heating_tolerance
    }

    pub fn cooling_tolerance(&self) -> ThermodynamicTemperature {
        self.cooling_tolerance
    }

    pub fn flow(&self) -> VolumeRate {
        self.pump.flow()
    }

    pub fn is_heating(&self) -> bool {
        self.heater.is_on()
    }

    pub fn should_pump(&self) -> bool {
        self.pump.is_requested()
    }

    pub fn cooling_mode(&self) -> Option<CoolingMode> {
        self.cooling.mode()
    }

    pub fn revolutions(&self) -> AngularVelocity {
        self.cooling.revolutions()
    }

    pub fn power(&self) -> f64 {
        self.heater.power(&self.config)
    }

    pub fn total_energy(&self) -> f64 {
        self.heater.total_energy()
    }

    // --- permissions and setpoints ---

    pub fn allow_pump(&mut self) {
        self.pump.allow();
    }

    pub fn allow_heating(&mut self) {
        self.heating_allowed = true;
    }

    pub fn disallow_heating(&mut self) {
        self.heating_allowed = false;
    }

    pub fn allow_cooling(&mut self) {
        self.cooling.allow();
    }

    /// Cuts cooling now and withdraws permission to restart it.
    pub fn disable_cooling(&mut self) {
        self.cooling.force_off(Instant::now());
        self.cooling.disallow();
    }

    pub fn set_should_pump(&mut self, should_pump: bool) {
        if self.pump.set_requested(should_pump) {
            self.reset_control(Instant::now(), ControlResetReason::PumpCommandChanged);
        }
    }

    pub fn set_target_temperature(&mut self, temperature: ThermodynamicTemperature) {
        self.reset_control(Instant::now(), ControlResetReason::TargetTemperatureChanged);
        self.target_temperature = temperature;
    }

    pub fn set_max_revolutions(&mut self, revolutions: AngularVelocity) {
        self.cooling.set_max_revolutions(revolutions);
    }

    pub fn set_heating_tolerance(&mut self, tolerance: ThermodynamicTemperature) {
        self.heating_tolerance = tolerance;
        self.reset_control(Instant::now(), ControlResetReason::HeatingToleranceChanged);
    }

    pub fn set_cooling_tolerance(&mut self, tolerance: ThermodynamicTemperature) {
        self.cooling_tolerance = tolerance;
        self.reset_control(Instant::now(), ControlResetReason::CoolingToleranceChanged);
    }

    // --- pid tuning ---

    pub fn pid_kp(&self) -> f64 {
        self.heater.kp()
    }

    pub fn pid_ki(&self) -> f64 {
        self.heater.ki()
    }

    pub fn pid_kd(&self) -> f64 {
        self.heater.kd()
    }

    pub fn set_pid_kp(&mut self, kp: f64) {
        self.heater
            .configure_pid(self.heater.ki(), kp, self.heater.kd());
        self.reset_control(Instant::now(), ControlResetReason::PidParametersChanged);
    }

    pub fn set_pid_ki(&mut self, ki: f64) {
        self.heater
            .configure_pid(ki, self.heater.kp(), self.heater.kd());
        self.reset_control(Instant::now(), ControlResetReason::PidParametersChanged);
    }

    pub fn set_pid_kd(&mut self, kd: f64) {
        self.heater
            .configure_pid(self.heater.ki(), self.heater.kp(), kd);
        self.reset_control(Instant::now(), ControlResetReason::PidParametersChanged);
    }

    // --- thermal safety timing ---

    pub fn thermal_flow_settle_duration(&self) -> Duration {
        self.config.thermal_flow_settle_duration
    }

    pub fn set_thermal_flow_settle_duration(&mut self, duration: Duration) {
        self.config.thermal_flow_settle_duration = duration;
    }

    pub fn pump_cooldown_min_temperature(&self) -> ThermodynamicTemperature {
        self.config.pump_cooldown_min_temperature
    }

    pub fn set_pump_cooldown_min_temperature(&mut self, temperature: ThermodynamicTemperature) {
        self.config.pump_cooldown_min_temperature = temperature;
    }

    pub fn pump_cooldown_remaining(&self, now: Instant) -> Duration {
        self.pump.cooldown_remaining(now, &self.config)
    }

    pub fn is_pump_cooldown_active(&self, now: Instant) -> bool {
        self.pump.is_cooling_down(now, &self.config)
    }

    pub fn heating_startup_wait_remaining(&self, now: Instant) -> Duration {
        self.pump.settle_remaining(now, &self.config)
    }

    /// The loop wants heat and has flow, but that flow has not been steady long enough yet.
    pub fn is_heating_startup_wait_active(&self, now: Instant) -> bool {
        self.heating_allowed
            && !self.heater.is_on()
            && self.error() > self.heating_tolerance.get::<degree_celsius>()
            && self.pump.is_delivering()
            && self.pump.has_thermal_flow(&self.config)
            && !self.pump.flow_is_settled(now, &self.config)
    }

    pub fn drain_notices(&mut self) -> Vec<ControllerNotice> {
        std::mem::take(&mut self.notices)
    }

    // --- control loop ---

    pub fn update(&mut self, now: Instant) {
        let dt = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;

        // The pump needs to know whether there is residual heat to flush before it may
        // stop, so this is sampled before the pump acts on the standby request.
        let heater_recently_active = self.heater.was_recently_active(now, &self.config);
        let outcome = self.pump.update(now, &self.config, heater_recently_active);
        if outcome.control_reset {
            self.reset_control(now, ControlResetReason::PumpCommandChanged);
        }
        if outcome.low_flow_trip {
            self.notices.push(ControllerNotice::PumpStoppedLowFlow);
        }

        self.current_temperature = self.temperature_sensor.read();
        self.enforce_hard_limits(now);

        let error = self.error();
        self.heater.tick(error, now, &self.config);

        // Heat may only ever be applied to a loop with proven, settled flow. This is
        // enforced ahead of the demand branch on purpose: losing flow has to cut the
        // element whatever the loop happens to want, and the demand branch is chosen by
        // temperature alone — flow can just as easily fail while sitting in the deadband.
        if !self.may_heat(now) {
            self.heater.force_off(now);
        }

        match self.demand(error) {
            Demand::Heat => self.run_heating(error, now, dt),
            Demand::Cool => self.run_cooling(now),
            Demand::Hold => self.run_deadband(now),
        }
    }

    fn error(&self) -> f64 {
        self.target_temperature.get::<degree_celsius>()
            - self.current_temperature.get::<degree_celsius>()
    }

    fn demand(&self, error: f64) -> Demand {
        if error > self.heating_tolerance.get::<degree_celsius>() {
            Demand::Heat
        } else if error < -self.cooling_tolerance.get::<degree_celsius>() {
            Demand::Cool
        } else {
            Demand::Hold
        }
    }

    /// Absolute bounds, independent of demand or permissions.
    fn enforce_hard_limits(&mut self, now: Instant) {
        if self.current_temperature < self.config.min_temperature && self.cooling.is_on() {
            self.cooling.force_off(now);
        } else if self.current_temperature > self.config.max_temperature && self.heater.is_on() {
            self.heater.force_off(now);
        }
    }

    /// Flow is present right now — enough to carry heat away from the fan side.
    fn cooling_interlock_ok(&self) -> bool {
        self.pump.has_thermal_flow(&self.config) && self.pump.is_delivering()
    }

    /// Permission plus flow that is present *and* has been steady long enough to trust
    /// the element to it. The single gate on energizing the heater.
    fn may_heat(&self, now: Instant) -> bool {
        self.heating_allowed
            && self.cooling_interlock_ok()
            && self.pump.flow_is_settled(now, &self.config)
    }

    fn run_heating(&mut self, error: f64, now: Instant, dt: f64) {
        self.cooling.request_off(now, &self.config);

        // The interlock already forced the element off for this cycle if it was unsafe.
        if self.may_heat(now) {
            self.heater.drive(error, now, dt, &self.config);
        }
    }

    fn run_cooling(&mut self, now: Instant) {
        self.heater.request(false, now, &self.config);

        if self.cooling.is_allowed() && self.cooling_interlock_ok() {
            let temp_offset = self.current_temperature.get::<degree_celsius>()
                - self.target_temperature.get::<degree_celsius>();
            self.cooling.drive(temp_offset, now, &self.config);
        } else {
            self.cooling.request_off(now, &self.config);
        }
    }

    fn run_deadband(&mut self, now: Instant) {
        self.heater.settle(now, &self.config);
        self.cooling.request_off(now, &self.config);
    }

    fn reset_control(&mut self, now: Instant, reason: ControlResetReason) {
        self.heater.reset_control(now);
        self.notices.push(ControllerNotice::ControlReset(reason));
    }
}
