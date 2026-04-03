use crate::aquapath1::{Flow, Temperature};
use control_core::controllers::pid::PidController;
use ethercat_hal::io::encoder_input::EncoderInput;
use ethercat_hal::io::{
    analog_output::AnalogOutput, digital_output::DigitalOutput, temperature_input::TemperatureInput,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use units::AngularVelocity;
use units::angular_velocity::revolution_per_minute;
use units::f64::{ThermodynamicTemperature, VolumeRate};
use units::thermodynamic_temperature::degree_celsius;
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
            pump_startup_grace_period: Duration::from_secs(3),
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
    last_update: Instant,
    temperature_pid_output: f64,
    pwm_period: Duration,
    last_heating_switch: Instant,
    last_cooling_switch: Instant,

    pub temperature: Temperature,
    pub target_temperature: ThermodynamicTemperature,
    pub current_temperature: ThermodynamicTemperature,
    pub temp_reservoir: ThermodynamicTemperature,
    pub min_temperature: ThermodynamicTemperature,
    pub max_temperature: ThermodynamicTemperature,

    pub cooling_controller: AnalogOutput,
    pub cooling_relais: DigitalOutput,

    pub cooling_tolerance: ThermodynamicTemperature,
    pub heating_tolerance: ThermodynamicTemperature,

    pub current_revolutions: AngularVelocity,
    pub max_revolutions: AngularVelocity,
    pub cooling_mode: Option<CoolingMode>,

    pub heating_relais: DigitalOutput,
    pub temperature_sensor_in: TemperatureInput,
    pub temperature_sensor_out: TemperatureInput,

    pub power: f64,
    pub total_energy: f64,

    pub cooling_allowed: bool,
    pub heating_allowed: bool,

    pub flow: Flow,
    pump_relais: DigitalOutput,
    pub should_pump: bool,
    pump_started_at: Option<Instant>,
    flow_became_valid_at: Option<Instant>,
    pump_cooldown_started_at: Option<Instant>,
    heating_last_active_at: Option<Instant>,
    pub flow_sensor: EncoderInput,
    pub pump_allowed: bool,
    pub current_flow: VolumeRate,
    pub max_flow: VolumeRate,
    config: ControllerConfig,
    pending_notices: Vec<ControllerNotice>,
}

impl Controller {
    fn reset_control_state(&mut self, now: Instant, reason: Option<ControlResetReason>) {
        self.pid.reset();
        self.temperature_pid_output = 0.0;
        self.window_start = now;
        if let Some(reason) = reason {
            self.pending_notices
                .push(ControllerNotice::ControlReset(reason));
        }
    }

    fn cooling_target_rpm(
        temp_offset: f64,
        max_rpm: f64,
        config: CoolingRampConfig,
    ) -> (f64, CoolingMode) {
        let full_band = config.full_band.get::<degree_celsius>();
        let near_band = config.near_band.get::<degree_celsius>();
        let tolerance = config.tolerance.get::<degree_celsius>();

        if temp_offset >= full_band {
            // Far above target: cool aggressively.
            return (max_rpm, CoolingMode::Max);
        }

        if temp_offset >= near_band {
            // Mid-range: ramp from 60% to 100% of max.
            let t = (temp_offset - near_band) / (full_band - near_band);
            return ((0.6 + 0.4 * t) * max_rpm, CoolingMode::Ramp);
        }

        // Near target (but above cooling tolerance): low, smooth cooling.
        // Ramp from COOLING_MIN_RPM to 60% of max to reduce overshoot/chatter.
        let t = ((temp_offset - tolerance) / (near_band - tolerance)).clamp(0.0, 1.0);
        let lower = config.min_rpm.min(max_rpm);
        let upper = 0.6 * max_rpm;
        (lower + (upper - lower) * t, CoolingMode::Low)
    }

    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        temp: Temperature,
        target_tempetature: ThermodynamicTemperature,
        cooling_controller: AnalogOutput,
        cooling_relais: DigitalOutput,
        heating_relais: DigitalOutput,
        temp_sensor_in: TemperatureInput,
        temp_sensor_out: TemperatureInput,
        max_revolutions: AngularVelocity,

        flow: Flow,
        pump_relais: DigitalOutput,
        flow_sensor: EncoderInput,
        config: ControllerConfig,
    ) -> Self {
        let now = Instant::now();
        Self {
            pid: PidController::new(kp, ki, kd),
            window_start: now,
            last_update: now,
            temperature_pid_output: 0.0,
            pwm_period: config.heating_pwm_period,
            last_heating_switch: now,
            last_cooling_switch: now,
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
            temperature_sensor_in: temp_sensor_in,
            temperature_sensor_out: temp_sensor_out,

            power: 0.0,
            total_energy: 0.0,

            flow,
            pump_relais,
            flow_sensor,
            should_pump: false,
            pump_started_at: None,
            flow_became_valid_at: None,
            pump_cooldown_started_at: None,
            heating_last_active_at: None,
            current_flow: VolumeRate::new::<liter_per_minute>(0.0),
            pump_allowed: false,
            max_flow: VolumeRate::new::<liter_per_minute>(10.0),
            config,
            pending_notices: Vec::new(),
        }
    }

    pub fn turn_pump_off(&mut self) {
        self.flow.pump = false;
        self.pump_started_at = None;
        self.pump_relais.set(false);
    }

    pub fn turn_pump_on(&mut self) {
        self.flow.pump = true;
        self.pump_started_at = Some(Instant::now());
        self.pump_relais.set(true);
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
        self.reset_control_state(
            Instant::now(),
            Some(ControlResetReason::TargetTemperatureChanged),
        );
        self.target_temperature = temperature;
    }

    pub fn get_temp_in(&mut self) -> ThermodynamicTemperature {
        let temp = self.temperature_sensor_in.get_temperature();
        match temp {
            Ok(value) => ThermodynamicTemperature::new::<degree_celsius>(value),
            Err(_) => ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }

    pub fn get_temp_out(&mut self) -> ThermodynamicTemperature {
        let temp = self.temperature_sensor_out.get_temperature();
        match temp {
            Ok(value) => ThermodynamicTemperature::new::<degree_celsius>(value),
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
        self.cooling_relais.set(true);
        self.temperature.cooling = true;
    }

    pub fn turn_cooling_off(&mut self) {
        self.cooling_relais.set(false);
        self.cooling_controller.set(0.0);
        self.current_revolutions = AngularVelocity::new::<revolution_per_minute>(0.0);
        self.cooling_mode = None;
        self.temperature.cooling = false;
    }

    pub fn turn_heating_on(&mut self) {
        self.heating_relais.set(true);
        self.temperature.heating = true;
        self.power = self.config.heating_element_power;
    }

    pub fn turn_heating_off(&mut self) {
        self.heating_relais.set(false);
        self.temperature.heating = false;
        self.power = 0.0;
    }

    pub fn set_should_pump(&mut self, should_pump: bool) {
        if self.should_pump != should_pump {
            self.reset_control_state(Instant::now(), Some(ControlResetReason::PumpCommandChanged));
        }
        self.should_pump = should_pump;
    }

    pub fn get_should_pump(&mut self) -> bool {
        self.should_pump
    }

    pub fn get_flow(&mut self) -> VolumeRate {
        let value = match self.flow_sensor.get_frequency_value() {
            Ok(val) => val,
            Err(_e) => {
                return VolumeRate::new::<liter_per_minute>(0.0);
            }
        };

        match value {
            Some(val) => {
                if val == 0 {
                    return VolumeRate::new::<liter_per_minute>(0.0);
                }
                // Formula: f = 8.1*q - 3, so q = (f + 3) / 8.1
                let actual_flow = ((val / 100) as f32 + 3.0) / 8.1;
                VolumeRate::new::<liter_per_minute>(actual_flow.into())
            }
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
        self.reset_control_state(
            Instant::now(),
            Some(ControlResetReason::CoolingToleranceChanged),
        );
    }

    pub fn set_heating_tolerance(&mut self, tolerance: ThermodynamicTemperature) {
        self.heating_tolerance = tolerance;
        self.reset_control_state(
            Instant::now(),
            Some(ControlResetReason::HeatingToleranceChanged),
        );
    }

    pub fn get_pid_kp(&self) -> f64 {
        self.pid.get_kp()
    }

    pub fn get_pid_ki(&self) -> f64 {
        self.pid.get_ki()
    }

    pub fn get_pid_kd(&self) -> f64 {
        self.pid.get_kd()
    }

    pub fn get_thermal_flow_settle_duration(&self) -> Duration {
        self.config.thermal_flow_settle_duration
    }

    pub fn set_thermal_flow_settle_duration(&mut self, duration: Duration) {
        self.config.thermal_flow_settle_duration = duration;
    }

    pub fn get_pump_cooldown_min_temperature(&self) -> ThermodynamicTemperature {
        self.config.pump_cooldown_min_temperature
    }

    pub fn set_pump_cooldown_min_temperature(&mut self, temperature: ThermodynamicTemperature) {
        self.config.pump_cooldown_min_temperature = temperature;
    }

    pub fn get_pump_cooldown_remaining(&self, now: Instant) -> Duration {
        match self.pump_cooldown_started_at {
            Some(started_at) => self
                .config
                .thermal_flow_settle_duration
                .saturating_sub(now.duration_since(started_at)),
            None => Duration::ZERO,
        }
    }

    pub fn is_pump_cooldown_active(&self, now: Instant) -> bool {
        self.pump_cooldown_started_at.is_some()
            && self.current_temperature.get::<degree_celsius>()
                > self
                    .config
                    .pump_cooldown_min_temperature
                    .get::<degree_celsius>()
            && !self.shared_thermal_delay_elapsed(self.pump_cooldown_started_at, now)
    }

    pub fn get_heating_startup_wait_remaining(&self, now: Instant) -> Duration {
        match self.flow_became_valid_at {
            Some(started_at) => self
                .config
                .thermal_flow_settle_duration
                .saturating_sub(now.duration_since(started_at)),
            None => self.config.thermal_flow_settle_duration,
        }
    }

    pub fn is_heating_startup_wait_active(&self, now: Instant) -> bool {
        let error = self.target_temperature.get::<degree_celsius>()
            - self.current_temperature.get::<degree_celsius>();
        let pump_is_running = self.flow.pump && self.should_pump;
        let has_flow_for_thermal = self.current_flow >= self.config.min_flow_for_thermal;

        self.heating_allowed
            && !self.temperature.heating
            && error > self.heating_tolerance.get::<degree_celsius>()
            && pump_is_running
            && has_flow_for_thermal
            && !self.shared_thermal_delay_elapsed(self.flow_became_valid_at, now)
    }

    pub fn set_pid_kp(&mut self, kp: f64) {
        self.pid.configure(self.pid.get_ki(), kp, self.pid.get_kd());
        self.reset_control_state(
            Instant::now(),
            Some(ControlResetReason::PidParametersChanged),
        );
    }

    pub fn set_pid_ki(&mut self, ki: f64) {
        self.pid.configure(ki, self.pid.get_kp(), self.pid.get_kd());
        self.reset_control_state(
            Instant::now(),
            Some(ControlResetReason::PidParametersChanged),
        );
    }

    pub fn set_pid_kd(&mut self, kd: f64) {
        self.pid.configure(self.pid.get_ki(), self.pid.get_kp(), kd);
        self.reset_control_state(
            Instant::now(),
            Some(ControlResetReason::PidParametersChanged),
        );
    }

    pub fn drain_notices(&mut self) -> Vec<ControllerNotice> {
        std::mem::take(&mut self.pending_notices)
    }

    fn pump_startup_grace_elapsed(&self, now: Instant) -> bool {
        match self.pump_started_at {
            Some(started_at) => {
                now.duration_since(started_at) >= self.config.pump_startup_grace_period
            }
            None => false,
        }
    }

    fn shared_thermal_delay_elapsed(&self, started_at: Option<Instant>, now: Instant) -> bool {
        match started_at {
            Some(started_at) => {
                now.duration_since(started_at) >= self.config.thermal_flow_settle_duration
            }
            None => false,
        }
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

        let has_flow_for_thermal = current_flow >= self.config.min_flow_for_thermal;

        if self.flow.pump
            && self.should_pump
            && !has_flow_for_thermal
            && self.pump_startup_grace_elapsed(now)
        {
            self.set_should_pump(false);
            self.pending_notices
                .push(ControllerNotice::PumpStoppedLowFlow);
        }

        let should_flow = self.get_should_pump();
        self.flow.should_pump = should_flow;

        self.current_temperature = self.get_temp_in();
        self.temp_reservoir = self.get_temp_out();

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
        } else if self.flow.pump && (!self.get_pump() || !should_keep_pump_running) {
            self.turn_pump_off();
            self.pump_cooldown_started_at = None;
        }

        let pump_is_running = self.flow.pump && self.should_pump;

        if pump_is_running && has_flow_for_thermal {
            if self.flow_became_valid_at.is_none() {
                self.flow_became_valid_at = Some(now);
            }
        } else {
            self.flow_became_valid_at = None;
        }

        if self.current_temperature < self.min_temperature && self.temperature.cooling {
            self.turn_cooling_off();
        } else if self.current_temperature > self.max_temperature && self.temperature.heating {
            self.turn_heating_off();
        }

        // Calculate PID error once
        let error = self.target_temperature.get::<degree_celsius>()
            - self.current_temperature.get::<degree_celsius>();

        let control = self.pid.update(error, now);
        self.temperature_pid_output = if control.is_finite() { control } else { 0.0 };

        let mut elapsed_in_window = now.duration_since(self.window_start);
        if elapsed_in_window >= self.pwm_period {
            self.window_start = now;
            elapsed_in_window = Duration::ZERO;
        }
        let flow_stable_long_enough =
            self.shared_thermal_delay_elapsed(self.flow_became_valid_at, now);
        let cooling_interlock_ok = has_flow_for_thermal && pump_is_running;
        let heating_interlock_ok = cooling_interlock_ok && flow_stable_long_enough;

        // Decide whether to heat or cool based on error
        if error > self.heating_tolerance.get::<degree_celsius>() {
            // Need heating (current < target)
            if self.temperature.cooling {
                self.set_cooling_state(false, now);
            }

            if self.heating_allowed && heating_interlock_ok {
                if error >= self.config.heating_full_power_error.get::<degree_celsius>() {
                    // Warmup phase: force full heating when far below target.
                    self.set_heating_state(true, now);
                } else {
                    let heating_duty = self.temperature_pid_output.clamp(0.0, 1.0);
                    let on_time = self.pwm_period.mul_f64(heating_duty);
                    let should_heat_on = heating_duty > 0.0 && elapsed_in_window < on_time;
                    self.set_heating_state(should_heat_on, now);
                }
                if self.temperature.heating {
                    self.heating_last_active_at = Some(now);
                    self.total_energy += self.get_current_power() * dt / 3600.0;
                }
            } else {
                // Flow/pump safety interlock or heating permission blocks heating.
                self.set_heating_state(false, now);
            }
        } else if error < -self.cooling_tolerance.get::<degree_celsius>() {
            // Need cooling (current > target)
            if self.temperature.heating {
                self.set_heating_state(false, now);
            }
            if self.cooling_allowed && cooling_interlock_ok {
                self.set_cooling_state(true, now);
                let max_revolutions = self.get_max_revolutions();
                let max_rpm = max_revolutions.get::<revolution_per_minute>();
                let temp_offset = self.current_temperature.get::<degree_celsius>()
                    - self.target_temperature.get::<degree_celsius>();

                let (target_revolutions, cooling_mode) =
                    Self::cooling_target_rpm(temp_offset, max_rpm, self.config.cooling);
                let target_revolutions = target_revolutions.clamp(0.0, max_rpm);

                if self.temperature.cooling {
                    self.cooling_controller
                        .set(target_revolutions as f32 / 10.0);
                    self.current_revolutions =
                        AngularVelocity::new::<revolution_per_minute>(target_revolutions);
                    self.cooling_mode = Some(cooling_mode);
                }
            } else {
                self.set_cooling_state(false, now);
            }
        } else {
            // Inside deadband: keep actuators off and clear PID memory to prevent windup.
            self.set_heating_state(false, now);
            self.set_cooling_state(false, now);
            self.temperature_pid_output = 0.0;
            self.pid.reset();
        }
    }
}
