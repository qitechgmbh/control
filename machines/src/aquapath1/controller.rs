use crate::aquapath1::VolumeRate;
use crate::aquapath1::{Flow, Temperature};
use control_core::controllers::pid::PidController;
use ethercat_hal::io::encoder_input::EncoderInput;
use ethercat_hal::io::{
    analog_output::AnalogOutput, digital_output::DigitalOutput, temperature_input::TemperatureInput,
};
use std::time::Instant;
use units::AngularVelocity;
use units::angular_velocity::revolution_per_minute;
use units::f64::ThermodynamicTemperature;
use units::thermodynamic_temperature::degree_celsius;
use units::volume_rate::liter_per_minute;

// Constants

const DEFAULT_TEMPERATURE_C: f64 = 25.0;
const DEFAULT_MIN_TEMP_C: f64 = 10.0;
const DEFAULT_MAX_TEMP_C: f64 = 50.0;
const DEFAULT_TOLERANCE_C: f64 = 2.0;
const DEFAULT_POWER_W: f64 = 700.0;
const DEFAULT_MAX_FLOW_LPM: f64 = 10.0;

// Flow sensor formula: f = 8.1*q - 3  =>  q = (f + 3) / 8.1
const FLOW_SENSOR_SCALE: f32 = 8.1;
const FLOW_SENSOR_OFFSET: f32 = 3.0;
const FLOW_SENSOR_DIVISOR: f32 = 100.0;

// Controller

#[derive(Debug)]
pub struct Controller {
    // Control
    pub pid: PidController,
    window_start: Instant,

    // Temperature state
    pub temperature: Temperature,
    pub target_temperature: ThermodynamicTemperature,
    pub current_temperature: ThermodynamicTemperature,
    pub temp_reservoir: ThermodynamicTemperature,
    pub min_temperature: ThermodynamicTemperature,
    pub max_temperature: ThermodynamicTemperature,
    pub cooling_tolerance: ThermodynamicTemperature,
    pub heating_tolerance: ThermodynamicTemperature,

    // Temperature hardware
    pub temperature_sensor_in: TemperatureInput,
    pub temperature_sensor_out: TemperatureInput,
    pub cooling_controller: AnalogOutput,
    pub cooling_relais: DigitalOutput,
    pub heating_relais_1: DigitalOutput,

    // Temperature permissions
    pub cooling_allowed: bool,
    pub heating_allowed: bool,

    // Cooling revolutions
    pub current_revolutions: AngularVelocity,
    pub max_revolutions: AngularVelocity,

    // Energy tracking
    pub power: f64,
    pub total_energy: f64,

    // Flow state
    pub flow: Flow,
    pub current_flow: VolumeRate,
    pub max_flow: VolumeRate,
    pub should_pump: bool,
    pub pump_allowed: bool,

    // Flow hardware
    pump_relais: DigitalOutput,
    pub flow_sensor: EncoderInput,
}

impl Controller {
    pub fn new(
        pid: PidController,
        temperature: Temperature,
        target_temperature: ThermodynamicTemperature,
        cooling_controller: AnalogOutput,
        cooling_relais: DigitalOutput,
        heating_relais_1: DigitalOutput,
        temperature_sensor_in: TemperatureInput,
        temperature_sensor_out: TemperatureInput,
        max_revolutions: AngularVelocity,
        flow: Flow,
        pump_relais: DigitalOutput,
        flow_sensor: EncoderInput,
    ) -> Self {
        let celsius = |c| ThermodynamicTemperature::new::<degree_celsius>(c);

        Self {
            pid,
            window_start: Instant::now(),

            temperature,
            target_temperature,
            current_temperature: celsius(DEFAULT_TEMPERATURE_C),
            temp_reservoir: celsius(DEFAULT_TEMPERATURE_C),
            min_temperature: celsius(DEFAULT_MIN_TEMP_C),
            max_temperature: celsius(DEFAULT_MAX_TEMP_C),
            cooling_tolerance: celsius(DEFAULT_TOLERANCE_C),
            heating_tolerance: celsius(DEFAULT_TOLERANCE_C),

            temperature_sensor_in,
            temperature_sensor_out,
            cooling_controller,
            cooling_relais,
            heating_relais_1,

            cooling_allowed: false,
            heating_allowed: false,

            current_revolutions: AngularVelocity::new::<revolution_per_minute>(0.0),
            max_revolutions,

            power: DEFAULT_POWER_W,
            total_energy: 0.0,

            flow,
            current_flow: VolumeRate::new::<liter_per_minute>(0.0),
            max_flow: VolumeRate::new::<liter_per_minute>(DEFAULT_MAX_FLOW_LPM),
            should_pump: false,
            pump_allowed: false,

            pump_relais,
            flow_sensor,
        }
    }

    // Update loop

    pub fn update(&mut self, now: Instant) {
        let elapsed = now.duration_since(self.window_start);
        self.window_start = now;

        self.update_flow();
        self.update_temperatures();
        self.enforce_safety_limits();
        self.update_thermal_control(now, elapsed.as_secs_f64());
    }

    fn update_flow(&mut self) {
        self.current_flow = self.read_flow();
        self.flow.flow = self.current_flow;
        self.flow.should_pump = self.should_pump;

        match (self.flow.pump, self.pump_allowed && self.should_pump) {
            (false, true) => self.turn_pump_on(),
            (true, false) => self.turn_pump_off(),
            _ => {}
        }
    }

    fn update_temperatures(&mut self) {
        self.current_temperature = self.read_temp_in();
        self.temp_reservoir = self.read_temp_out();
    }

    fn enforce_safety_limits(&mut self) {
        if self.current_temperature < self.min_temperature && self.temperature.cooling {
            self.turn_cooling_off();
        } else if self.current_temperature > self.max_temperature && self.temperature.heating {
            self.turn_heating_off();
        }
    }

    fn update_thermal_control(&mut self, now: Instant, elapsed_secs: f64) {
        // This keeps the deadband logic independent of PID gain tuning.
        let error = self.target_temperature.get::<degree_celsius>()
            - self.current_temperature.get::<degree_celsius>();

        // Not used for switching decisions; only for how hard to heat/cool.
        let pid_output = self.pid.update(error, now);

        let has_flow = self.current_flow > VolumeRate::new::<liter_per_minute>(0.0);

        let heating_deadband = self.heating_tolerance.get::<degree_celsius>();
        let cooling_deadband = self.cooling_tolerance.get::<degree_celsius>();

        if error > heating_deadband {
            // Temperature is below target by more than the tolerance -> heat
            self.handle_heating(has_flow, elapsed_secs);
        } else if error < -cooling_deadband {
            // Temperature is above target by more than the tolerance -> cool
            self.handle_cooling(has_flow, pid_output.abs());
        } else {
            // Inside deadband -> idle, actively shut everything off to prevent chatter
            if self.temperature.heating {
                self.turn_heating_off();
            }
            if self.temperature.cooling {
                self.turn_cooling_off();
            }
        }
    }

    fn handle_heating(&mut self, has_flow: bool, elapsed_secs: f64) {
        if self.temperature.cooling {
            self.turn_cooling_off();
        }

        if self.heating_allowed && has_flow {
            self.turn_heating_on();
            // Heating is bang-bang (digital relay), so the PID only determines on/off.
            // Energy is tracked using the fixed rated power.
            self.total_energy += self.power * elapsed_secs / 3600.0;
        } else if self.temperature.heating {
            // No flow or heating not allowed — safety cutoff
            self.turn_heating_off();
        }
    }

    fn handle_cooling(&mut self, has_flow: bool, pid_output: f64) {
        if self.temperature.heating {
            self.turn_heating_off();
        }

        if self.cooling_allowed && has_flow {
            if !self.temperature.cooling {
                self.turn_cooling_on();
            }

            let max_rpm = self.max_revolutions.get::<revolution_per_minute>();
            // PID output magnitude drives compressor RPM.
            // Gains in PidController::new() control the scaling between error and RPM.
            let target_rpm = pid_output.clamp(0.0, max_rpm);
            self.cooling_controller.set(target_rpm as f32 / 10.0);
            self.current_revolutions = AngularVelocity::new::<revolution_per_minute>(target_rpm);
        } else if self.temperature.cooling {
            self.turn_cooling_off();
        }
    }

    // Hardware reads

    fn read_temp_in(&mut self) -> ThermodynamicTemperature {
        self.temperature_sensor_in
            .get_temperature()
            .map(|v| ThermodynamicTemperature::new::<degree_celsius>(v))
            .unwrap_or_else(|_| ThermodynamicTemperature::new::<degree_celsius>(0.0))
    }

    fn read_temp_out(&mut self) -> ThermodynamicTemperature {
        self.temperature_sensor_out
            .get_temperature()
            .map(|v| ThermodynamicTemperature::new::<degree_celsius>(v))
            .unwrap_or_else(|_| ThermodynamicTemperature::new::<degree_celsius>(0.0))
    }

    fn read_flow(&mut self) -> VolumeRate {
        let zero = VolumeRate::new::<liter_per_minute>(0.0);

        let raw = match self.flow_sensor.get_frequency_value() {
            Ok(Some(v)) if v != 0 => v,
            _ => return zero,
        };

        let flow_lpm = (raw as f32 / FLOW_SENSOR_DIVISOR + FLOW_SENSOR_OFFSET) / FLOW_SENSOR_SCALE;
        VolumeRate::new::<liter_per_minute>(flow_lpm.into())
    }

    // Pump control

    pub fn turn_pump_on(&mut self) {
        self.flow.pump = true;
        self.pump_relais.set(true);
    }

    pub fn turn_pump_off(&mut self) {
        self.flow.pump = false;
        self.pump_relais.set(false);
    }

    pub fn allow_pump(&mut self) {
        self.pump_allowed = true;
    }

    pub fn disallow_pump(&mut self) {
        self.pump_allowed = false;
    }

    // Cooling control

    pub fn turn_cooling_on(&mut self) {
        self.cooling_relais.set(true);
        self.temperature.cooling = true;
    }

    pub fn turn_cooling_off(&mut self) {
        self.cooling_relais.set(false);
        self.temperature.cooling = false;
        self.current_revolutions = AngularVelocity::new::<revolution_per_minute>(0.0);
    }

    pub fn allow_cooling(&mut self) {
        self.cooling_allowed = true;
    }

    pub fn disallow_cooling(&mut self) {
        self.cooling_allowed = false;
    }

    /// Turns cooling on and marks it as allowed.
    pub fn enable_cooling(&mut self) {
        self.allow_cooling();
        self.turn_cooling_on();
    }

    /// Turns cooling off and marks it as disallowed.
    pub fn disable_cooling(&mut self) {
        self.turn_cooling_off();
        self.disallow_cooling();
    }

    // Heating control

    pub fn turn_heating_on(&mut self) {
        self.heating_relais_1.set(true);
        self.temperature.heating = true;
    }

    pub fn turn_heating_off(&mut self) {
        self.heating_relais_1.set(false);
        self.temperature.heating = false;
        self.power = 0.0;
    }

    pub fn allow_heating(&mut self) {
        self.heating_allowed = true;
    }

    pub fn disallow_heating(&mut self) {
        self.heating_allowed = false;
    }

    /// Turns heating on and marks it as allowed.
    pub fn enable_heating(&mut self) {
        self.allow_heating();
        self.turn_heating_on();
    }

    /// Turns heating off and marks it as disallowed.
    pub fn disable_heating(&mut self) {
        self.turn_heating_off();
        self.disallow_heating();
    }

    // Configuration

    pub fn set_target_temperature(&mut self, temperature: ThermodynamicTemperature) {
        self.pid.reset();
        self.target_temperature = temperature;
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

    // Getters

    pub fn current_revolutions(&self) -> AngularVelocity {
        self.current_revolutions
    }

    pub fn max_revolutions(&self) -> AngularVelocity {
        self.max_revolutions
    }

    pub fn current_power(&self) -> f64 {
        self.power
    }

    pub fn total_energy(&self) -> f64 {
        self.total_energy
    }
}
