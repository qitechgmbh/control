use crate::aquapath1::{Flow, Temperature};
use control_core::controllers::pid::PidController;
use ethercat_hal::io::as006::{As006Flow, As006Temp};
use ethercat_hal::io::{analog_output::AnalogOutput, digital_output::DigitalOutput};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use units::AngularVelocity;
use units::angular_velocity::revolution_per_minute;
use units::f64::ThermodynamicTemperature;
use units::thermodynamic_temperature::{degree_celsius, kelvin};
use units::volume_rate::liter_per_minute;

#[derive(Debug, Clone, Copy)]
pub enum ControlResetReason {
    TargetTemperatureChanged,
    HeatingToleranceChanged,
    CoolingToleranceChanged,
    PidParametersChanged,
    PumpCommandChanged,
}

#[derive(Debug, Clone, Copy)]
pub enum ControllerNotice {
    ControlReset(ControlResetReason),
    PumpStoppedLowFlow,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CoolingMode {
    Low,
    Ramp,
    Max,
}

#[derive(Debug, Clone, Copy)]
pub struct CoolingRampConfig {
    pub tolerance: ThermodynamicTemperature,
    pub near_band: ThermodynamicTemperature,
    pub full_band: ThermodynamicTemperature,
    pub min_rpm: f64,
}

impl Default for CoolingRampConfig {
    fn default() -> Self {
        Self {
            tolerance: ThermodynamicTemperature::new::<degree_celsius>(0.8),
            near_band: ThermodynamicTemperature::new::<degree_celsius>(2.0),
            full_band: ThermodynamicTemperature::new::<degree_celsius>(4.0),
            min_rpm: 20.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ControllerConfig {
    pub min_flow_for_thermal: VolumeRate,
    pub pump_startup_grace_period: Duration,
    pub thermal_flow_settle_duration: Duration,
    pub heating_element_power: f64,
    pub heating_pwm_period: Duration,
    pub relay_min_on_time: Duration,
    pub relay_min_off_time: Duration,
    pub heating_full_power_error: ThermodynamicTemperature,
    pub heating_tolerance: ThermodynamicTemperature,
    pub pump_cooldown_min_temperature: ThermodynamicTemperature,
    pub cooling: CoolingRampConfig,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            min_flow_for_thermal: VolumeRate::new::<liter_per_minute>(0.2),
            pump_startup_grace_period: Duration::from_secs(15),
            thermal_flow_settle_duration: Duration::from_secs(10),
            heating_element_power: 700.0,
            heating_pwm_period: Duration::from_secs(12),
            relay_min_on_time: Duration::from_secs(5),
            relay_min_off_time: Duration::from_secs(5),
            heating_full_power_error: ThermodynamicTemperature::new::<degree_celsius>(4.0),
            heating_tolerance: ThermodynamicTemperature::new::<degree_celsius>(0.4),
            pump_cooldown_min_temperature: ThermodynamicTemperature::new::<degree_celsius>(45.0),
            cooling: CoolingRampConfig::default(),
        }
    }
}

#[derive(Debug)]
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

    pub heating_relais: DigitalOutput,
    pub temperature_sensor: As006Temp,

    pub power: f64,
    pub total_energy: f64,

    pub cooling_allowed: bool,
    pub heating_allowed: bool,
    pub should_pump: bool,
    pump_started_at: Option<Instant>,
    flow_became_valid_at: Option<Instant>,
    pump_cooldown_started_at: Option<Instant>,
    heating_last_active_at: Option<Instant>,
    pub flow_sensor: As006Flow,
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
    ) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd),
            window_start: Instant::now(),
            target_temperature: target_tempetature,
            current_temperature: ThermodynamicTemperature::new::<degree_celsius>(25.0),
            temp_reservoir: ThermodynamicTemperature::new::<degree_celsius>(25.0),
            min_temperature: ThermodynamicTemperature::new::<degree_celsius>(10.0),
            max_temperature: ThermodynamicTemperature::new::<degree_celsius>(80.0),
            temperature: temp,
            cooling_controller,
            cooling_tolerance: config.cooling.tolerance,
            heating_tolerance: config.heating_tolerance,
            current_revolutions: AngularVelocity::new::<revolution_per_minute>(0.0),
            max_revolutions,
            cooling_mode: None,
            cooling_relais,
            heating_relais,
            cooling_allowed: false,
            heating_allowed: false,
            temperature_sensor,
            power: 0.0,
            total_energy: 0.0,
            flow,
            pump_relais,
            flow_sensor,
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

    pub fn get_flow(&self) -> VolumeRate {
        match self.flow_sensor.get_flow_lpm() {
            Some(lpm) => VolumeRate::new::<liter_per_minute>(lpm),
            None => VolumeRate::new::<liter_per_minute>(0.0),
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

    fn can_switch_relay(&self, currently_on: bool, last_switch: Instant, now: Instant) -> bool {
        let min_dwell = if currently_on {
            self.config.relay_min_on_time
        } else {
            self.config.relay_min_off_time
        };
        now.duration_since(last_switch) >= min_dwell
    }

    fn set_heating_state(&mut self, on: bool, now: Instant) {
        if on == self.temperature.heating {
            return;
        }
        if !self.can_switch_relay(self.temperature.heating, self.last_heating_switch, now) {
            return;
        }

        if on {
            self.turn_heating_on();
        } else {
            self.turn_heating_off();
        }
        self.last_heating_switch = now;
    }

    fn set_cooling_state(&mut self, on: bool, now: Instant) {
        if on == self.temperature.cooling {
            return;
        }
        if !self.can_switch_relay(self.temperature.cooling, self.last_cooling_switch, now) {
            return;
        }

        if on {
            self.turn_cooling_on();
        } else {
            self.turn_cooling_off();
        }
        self.last_cooling_switch = now;
    }

    pub fn update(&mut self, now: Instant) {
        let dt = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;

        let current_flow = self.get_flow();
        self.current_flow = current_flow;
        self.flow.flow = current_flow;

        let should_flow = self.get_should_pump();
        self.flow.should_pump = should_flow;

        self.current_temperature = self.get_temp_in();
        self.temp_reservoir = self.current_temperature;

        let pump_cooldown_min_temperature = self.config.pump_cooldown_min_temperature;
        let pump_is_still_hot = self.current_temperature.get::<degree_celsius>()
            > pump_cooldown_min_temperature.get::<degree_celsius>();
        let heater_was_recently_active = self.temperature.heating
            || self.heating_last_active_at.is_some_and(|started_at| {
                now.duration_since(started_at) < self.config.thermal_flow_settle_duration
            });
        if !should_flow && self.flow.pump && heater_was_recently_active && pump_is_still_hot {
            if self.pump_cooldown_started_at.is_none() {
                self.pump_cooldown_started_at = Some(now);
            }
        } else if !self.flow.pump
            || (should_flow
                && !(self.pump_cooldown_started_at.is_some_and(|started_at| {
                    !self.shared_thermal_delay_elapsed(Some(started_at), now)
                }) && pump_is_still_hot))
        {
            self.pump_cooldown_started_at = None;
        }

        let cooldown_window_still_open = self
            .pump_cooldown_started_at
            .is_some_and(|started_at| !self.shared_thermal_delay_elapsed(Some(started_at), now));

        let pump_cooldown_active = self.pump_cooldown_started_at.is_some()
            && pump_is_still_hot
            && cooldown_window_still_open;
        let should_keep_pump_running = should_flow || pump_cooldown_active;

        if !self.flow.pump && self.get_pump() && should_keep_pump_running {
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
