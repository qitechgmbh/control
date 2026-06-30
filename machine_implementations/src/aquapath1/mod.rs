use crate::{
    MACHINE_AQUAPATH_V1, MachineMessage, VENDOR_QITECH,
    aquapath1::{
        api::{
            AquaPathV1Events, AquaPathV1Namespace, CoolingModeState, CoolingModeStates, FanState,
            FanStates, FlowState, FlowStates, LiveValuesEvent, ModeState, NoticeEvent, PidState,
            PidStates, StateEvent, TempState, TempStates, ThermalSafetyState, ThermalSafetyStates,
        },
        controller::{ControlResetReason, Controller, ControllerNotice},
    },
};
use api::{ToleranceState, ToleranceStates};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    machines::{MachineIdentification, MachineIdentificationUnique},
    units::{
        AngularVelocity, ThermodynamicTemperature, VolumeRate,
        angular_velocity::revolution_per_minute, thermodynamic_temperature::degree_celsius,
        volume_rate::liter_per_minute,
    },
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod controller;
pub mod new;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum AquaPathV1Mode {
    Standby,
    Auto,
}

pub enum AquaPathSideType {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Temperature {
    pub temperature: ThermodynamicTemperature,
    pub cooling: bool,
    pub heating: bool,
    pub target_temperature: ThermodynamicTemperature,
}

impl Default for Temperature {
    fn default() -> Self {
        Self {
            temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
            cooling: false,
            heating: false,
            target_temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    pub flow: VolumeRate,
    pub pump: bool,
    pub should_pump: bool,
}

impl Default for Flow {
    fn default() -> Self {
        Self {
            flow: VolumeRate::new::<liter_per_minute>(0.0),
            pump: false,
            should_pump: false,
        }
    }
}

pub struct AquaPathV1 {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: AquaPathV1Namespace,
    mode: AquaPathV1Mode,
    ambient_temperature_calibration: ThermodynamicTemperature,
    last_measurement_emit: Instant,
    left_controller: Controller,
    right_controller: Controller,
}

impl AquaPathV1 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_AQUAPATH_V1,
    };

    // in °C
    pub const DEFAULT_HEATING_TOLERANCE: f64 = 0.4;
    // in °C
    pub const DEFAULT_COOLING_TOLERANCE: f64 = 0.8;
    pub const DEFAULT_PID_KP: f64 = 0.16;
    pub const DEFAULT_PID_KI: f64 = 0.02;
    pub const DEFAULT_PID_KD: f64 = 0.0;
    // in s
    pub const THERMAL_FLOW_SETTLE_DURATION_MIN: f64 = 0.0;
    // in s
    pub const THERMAL_FLOW_SETTLE_DURATION_MAX: f64 = 30.0;
    // in °C
    pub const PUMP_COOLDOWN_MIN_TEMPERATURE_MIN: f64 = 10.0;
    // in °C
    pub const PUMP_COOLDOWN_MIN_TEMPERATURE_MAX: f64 = 80.0;

    pub fn get_live_values(&self) -> LiveValuesEvent {
        let now = Instant::now();
        LiveValuesEvent {
            left_temperature: self
                .left_controller
                .current_temperature
                .get::<degree_celsius>(),
            right_temperature: self
                .right_controller
                .current_temperature
                .get::<degree_celsius>(),
            left_flow: self.left_controller.current_flow.get::<liter_per_minute>(),
            right_flow: self.right_controller.current_flow.get::<liter_per_minute>(),
            left_revolutions: self
                .left_controller
                .current_revolutions
                .get::<revolution_per_minute>(),
            right_revolutions: self
                .right_controller
                .current_revolutions
                .get::<revolution_per_minute>(),
            left_power: self.left_controller.get_current_power(),
            right_power: self.right_controller.get_current_power(),
            left_heating: self.left_controller.temperature.heating,
            right_heating: self.right_controller.temperature.heating,
            left_cooling_mode: self.left_controller.cooling_mode,
            right_cooling_mode: self.right_controller.cooling_mode,
            left_pump_cooldown_active: self.left_controller.is_pump_cooldown_active(now),
            right_pump_cooldown_active: self.right_controller.is_pump_cooldown_active(now),
            left_pump_cooldown_remaining: self
                .left_controller
                .get_pump_cooldown_remaining(now)
                .as_secs_f64(),
            right_pump_cooldown_remaining: self
                .right_controller
                .get_pump_cooldown_remaining(now)
                .as_secs_f64(),
            left_heating_startup_wait_active: self
                .left_controller
                .is_heating_startup_wait_active(now),
            right_heating_startup_wait_active: self
                .right_controller
                .is_heating_startup_wait_active(now),
            left_heating_startup_wait_remaining: self
                .left_controller
                .get_heating_startup_wait_remaining(now)
                .as_secs_f64(),
            right_heating_startup_wait_remaining: self
                .right_controller
                .get_heating_startup_wait_remaining(now)
                .as_secs_f64(),
            left_total_energy: self.left_controller.get_total_energy(),
            right_total_energy: self.right_controller.get_total_energy(),
        }
    }

    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace.emit(AquaPathV1Events::LiveValues(event));
    }

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            is_default_state: false,
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            ambient_temperature_calibration: self
                .ambient_temperature_calibration
                .get::<degree_celsius>(),
            default_heating_tolerance: Self::DEFAULT_HEATING_TOLERANCE,
            default_cooling_tolerance: Self::DEFAULT_COOLING_TOLERANCE,
            default_pid_kp: Self::DEFAULT_PID_KP,
            default_pid_ki: Self::DEFAULT_PID_KI,
            default_pid_kd: Self::DEFAULT_PID_KD,
            temperature_states: TempStates {
                left: TempState {
                    temperature: self
                        .left_controller
                        .current_temperature
                        .get::<degree_celsius>(),

                    target_temperature: self
                        .left_controller
                        .target_temperature
                        .get::<degree_celsius>(),
                },
                right: TempState {
                    temperature: self
                        .right_controller
                        .current_temperature
                        .get::<degree_celsius>(),
                    target_temperature: self
                        .right_controller
                        .target_temperature
                        .get::<degree_celsius>(),
                },
            },
            flow_states: FlowStates {
                left: FlowState {
                    flow: self.left_controller.current_flow.get::<liter_per_minute>(),
                    should_flow: self.left_controller.should_pump,
                },
                right: FlowState {
                    flow: self.right_controller.current_flow.get::<liter_per_minute>(),
                    should_flow: self.right_controller.should_pump,
                },
            },
            fan_states: FanStates {
                left: FanState {
                    revolutions: self
                        .left_controller
                        .max_revolutions
                        .get::<revolution_per_minute>(),
                    max_revolutions: self
                        .left_controller
                        .max_revolutions
                        .get::<revolution_per_minute>(),
                },
                right: FanState {
                    revolutions: self
                        .right_controller
                        .max_revolutions
                        .get::<revolution_per_minute>(),
                    max_revolutions: self
                        .right_controller
                        .max_revolutions
                        .get::<revolution_per_minute>(),
                },
            },
            cooling_mode_states: CoolingModeStates {
                left: CoolingModeState {
                    mode: self.left_controller.cooling_mode,
                },
                right: CoolingModeState {
                    mode: self.right_controller.cooling_mode,
                },
            },
            tolerance_states: ToleranceStates {
                left: ToleranceState {
                    heating: self
                        .left_controller
                        .heating_tolerance
                        .get::<degree_celsius>(),
                    cooling: self
                        .left_controller
                        .cooling_tolerance
                        .get::<degree_celsius>(),
                },
                right: ToleranceState {
                    heating: self
                        .right_controller
                        .heating_tolerance
                        .get::<degree_celsius>(),
                    cooling: self
                        .right_controller
                        .cooling_tolerance
                        .get::<degree_celsius>(),
                },
            },
            pid_states: PidStates {
                left: PidState {
                    kp: self.left_controller.get_pid_kp(),
                    ki: self.left_controller.get_pid_ki(),
                    kd: self.left_controller.get_pid_kd(),
                },
                right: PidState {
                    kp: self.right_controller.get_pid_kp(),
                    ki: self.right_controller.get_pid_ki(),
                    kd: self.right_controller.get_pid_kd(),
                },
            },
            thermal_safety_states: ThermalSafetyStates {
                left: ThermalSafetyState {
                    thermal_delay: self
                        .left_controller
                        .get_thermal_flow_settle_duration()
                        .as_secs_f64(),
                    cooldown_min_temperature: self
                        .left_controller
                        .get_pump_cooldown_min_temperature()
                        .get::<degree_celsius>(),
                },
                right: ThermalSafetyState {
                    thermal_delay: self
                        .right_controller
                        .get_thermal_flow_settle_duration()
                        .as_secs_f64(),
                    cooldown_min_temperature: self
                        .right_controller
                        .get_pump_cooldown_min_temperature()
                        .get::<degree_celsius>(),
                },
            },
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(AquaPathV1Events::State(event));
    }

    pub fn emit_notice(&mut self, title: impl Into<String>, message: impl Into<String>) {
        let event = NoticeEvent {
            title: title.into(),
            message: message.into(),
        }
        .build();
        self.namespace.emit(AquaPathV1Events::Notice(event));
    }

    fn emit_controller_notice(&mut self, side_label: &str, notice: ControllerNotice) {
        match notice {
            ControllerNotice::ControlReset(reason) => {
                let message = match reason {
                    ControlResetReason::TargetTemperatureChanged => {
                        "Target temperature changed. PID control state and heater PWM timing were reset."
                    }
                    ControlResetReason::HeatingToleranceChanged => {
                        "Heating tolerance changed. PID control state and heater PWM timing were reset."
                    }
                    ControlResetReason::CoolingToleranceChanged => {
                        "Cooling tolerance changed. PID control state and heater PWM timing were reset."
                    }
                    ControlResetReason::PidParametersChanged => {
                        "PID settings changed. PID control state and heater PWM timing were reset."
                    }
                    ControlResetReason::PumpCommandChanged => {
                        "Pump command changed. PID control state and heater PWM timing were reset."
                    }
                };

                self.emit_notice(format!("{side_label}: Thermal Control Reset"), message);
            }
            ControllerNotice::PumpStoppedLowFlow => {
                self.emit_notice(
                    format!("{side_label}: Pump Turned Off"),
                    "Flow fell below the minimum thermal threshold while the pump was enabled. The pump was turned off and PID control state was reset.",
                );
            }
        }
    }

    pub const PID_MIN: f64 = 0.0;
    pub const PID_MAX: f64 = 5.0;
    // in °C
    pub const TOLERANCE_MIN: f64 = 0.0;
    // in °C
    pub const TOLERANCE_MAX: f64 = 10.0;

    fn mode_allows_standby_only_config(&self) -> bool {
        self.mode == AquaPathV1Mode::Standby
    }
}

impl std::fmt::Display for AquaPathV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Aquapath")
    }
}
impl AquaPathV1 {
    fn request_pump_off(&mut self) {
        self.left_controller.set_should_pump(false);
        self.right_controller.set_should_pump(false);
    }

    fn turn_cooling_off(&mut self) {
        self.left_controller.disable_cooling();
        self.right_controller.disable_cooling();
    }

    fn turn_cooling_on(&mut self) {
        self.left_controller.allow_cooling();
        self.right_controller.allow_cooling();
    }

    fn turn_heating_off(&mut self) {
        self.left_controller.disallow_heating();
        self.right_controller.disallow_heating();
    }

    fn turn_heating_on(&mut self) {
        self.left_controller.allow_heating();
        self.right_controller.allow_heating();
    }

    fn turn_pump_on(&mut self) {
        self.left_controller.allow_pump();
        self.right_controller.allow_pump();
    }

    fn turn_off_all(&mut self) {
        self.turn_cooling_off();
        self.turn_heating_off();
        self.request_pump_off();
    }

    fn turn_on_all(&mut self) {
        self.turn_cooling_on();
        self.turn_heating_on();
        self.turn_pump_on();
    }
    // Turn all OFF and do nothing
    fn switch_to_standby(&mut self) {
        match self.mode {
            AquaPathV1Mode::Standby => (),
            AquaPathV1Mode::Auto => self.turn_off_all(),
        };
        self.mode = AquaPathV1Mode::Standby;
    }

    fn switch_to_auto(&mut self) {
        match self.mode {
            AquaPathV1Mode::Auto => (),
            AquaPathV1Mode::Standby => self.turn_on_all(),
        }
        self.mode = AquaPathV1Mode::Auto;
    }

    fn switch_mode(&mut self, mode: AquaPathV1Mode) {
        if self.mode == mode {
            return;
        }
        match mode {
            AquaPathV1Mode::Standby => self.switch_to_standby(),
            AquaPathV1Mode::Auto => self.switch_to_auto(),
        }
    }
}

impl AquaPathV1 {
    fn set_mode_state(&mut self, mode: AquaPathV1Mode) {
        self.switch_mode(mode.clone());
        self.emit_state();
    }
}

impl AquaPathV1 {
    fn get_min_settable_temperature(&self) -> ThermodynamicTemperature {
        let ambient = self.ambient_temperature_calibration.get::<degree_celsius>();
        let min = self.left_controller.min_temperature.get::<degree_celsius>();
        let max = self.left_controller.max_temperature.get::<degree_celsius>();
        ThermodynamicTemperature::new::<degree_celsius>(ambient.max(min).min(max))
    }

    fn set_ambient_temperature_calibration(&mut self, ambient_temperature: f64) {
        let clamped = ambient_temperature
            .max(self.left_controller.min_temperature.get::<degree_celsius>())
            .min(self.left_controller.max_temperature.get::<degree_celsius>());
        self.ambient_temperature_calibration =
            ThermodynamicTemperature::new::<degree_celsius>(clamped);

        // Enforce the calibrated minimum immediately on already-configured targets.
        let min_settable = self.get_min_settable_temperature();
        let min_settable_temperature = min_settable.get::<degree_celsius>();

        if self
            .left_controller
            .target_temperature
            .get::<degree_celsius>()
            < min_settable_temperature
        {
            self.left_controller.set_target_temperature(min_settable);
        }
        if self
            .right_controller
            .target_temperature
            .get::<degree_celsius>()
            < min_settable_temperature
        {
            self.right_controller.set_target_temperature(min_settable);
        }

        self.emit_state();
    }

    fn set_target_temperature(&mut self, temperature: f64, cooling_type: AquaPathSideType) {
        let min_settable = self.get_min_settable_temperature().get::<degree_celsius>();
        let max_settable = self.left_controller.max_temperature.get::<degree_celsius>();
        let clamped_target = temperature.max(min_settable).min(max_settable);
        let target_temp = ThermodynamicTemperature::new::<degree_celsius>(clamped_target);

        match cooling_type {
            AquaPathSideType::Right => self.right_controller.set_target_temperature(target_temp),
            AquaPathSideType::Left => self.left_controller.set_target_temperature(target_temp),
        }
        self.emit_state();
    }

    fn set_should_pump(&mut self, should_pump: bool, cooling_type: AquaPathSideType) {
        match cooling_type {
            AquaPathSideType::Right => self.right_controller.set_should_pump(should_pump),
            AquaPathSideType::Left => self.left_controller.set_should_pump(should_pump),
        }
        self.emit_state();
    }
}

impl AquaPathV1 {
    fn set_max_revolutions(&mut self, revolutions: f64, fan_type: AquaPathSideType) {
        match fan_type {
            AquaPathSideType::Right => self
                .right_controller
                .set_max_revolutions(AngularVelocity::new::<revolution_per_minute>(revolutions)),
            AquaPathSideType::Left => self
                .left_controller
                .set_max_revolutions(AngularVelocity::new::<revolution_per_minute>(revolutions)),
        }
        self.emit_state();
    }
}

impl AquaPathV1 {
    fn sanitize_clamped(value: f64, min: f64, max: f64, fallback: f64) -> f64 {
        if !value.is_finite() {
            return fallback;
        }
        value.clamp(min, max)
    }

    fn set_heating_tolerance(&mut self, tolerance: f64, tolerance_type: AquaPathSideType) {
        match tolerance_type {
            AquaPathSideType::Right => {
                let fallback = self
                    .right_controller
                    .heating_tolerance
                    .get::<degree_celsius>();
                let value = Self::sanitize_clamped(
                    tolerance,
                    Self::TOLERANCE_MIN,
                    Self::TOLERANCE_MAX,
                    fallback,
                );
                self.right_controller
                    .set_heating_tolerance(ThermodynamicTemperature::new::<degree_celsius>(value))
            }
            AquaPathSideType::Left => {
                let fallback = self
                    .left_controller
                    .heating_tolerance
                    .get::<degree_celsius>();
                let value = Self::sanitize_clamped(
                    tolerance,
                    Self::TOLERANCE_MIN,
                    Self::TOLERANCE_MAX,
                    fallback,
                );
                self.left_controller
                    .set_heating_tolerance(ThermodynamicTemperature::new::<degree_celsius>(value))
            }
        }

        self.emit_state();
    }

    fn set_cooling_tolerance(&mut self, tolerance: f64, tolerance_type: AquaPathSideType) {
        match tolerance_type {
            AquaPathSideType::Right => {
                let fallback = self
                    .right_controller
                    .cooling_tolerance
                    .get::<degree_celsius>();
                let value = Self::sanitize_clamped(
                    tolerance,
                    Self::TOLERANCE_MIN,
                    Self::TOLERANCE_MAX,
                    fallback,
                );
                self.right_controller
                    .set_cooling_tolerance(ThermodynamicTemperature::new::<degree_celsius>(value))
            }
            AquaPathSideType::Left => {
                let fallback = self
                    .left_controller
                    .cooling_tolerance
                    .get::<degree_celsius>();
                let value = Self::sanitize_clamped(
                    tolerance,
                    Self::TOLERANCE_MIN,
                    Self::TOLERANCE_MAX,
                    fallback,
                );
                self.left_controller
                    .set_cooling_tolerance(ThermodynamicTemperature::new::<degree_celsius>(value))
            }
        }

        self.emit_state();
    }

    fn set_pid_kp(&mut self, value: f64, side: AquaPathSideType) {
        let current = match side {
            AquaPathSideType::Right => self.right_controller.get_pid_kp(),
            AquaPathSideType::Left => self.left_controller.get_pid_kp(),
        };
        let value = Self::sanitize_clamped(value, Self::PID_MIN, Self::PID_MAX, current);
        match side {
            AquaPathSideType::Right => self.right_controller.set_pid_kp(value),
            AquaPathSideType::Left => self.left_controller.set_pid_kp(value),
        }
        self.emit_state();
    }

    fn set_pid_ki(&mut self, value: f64, side: AquaPathSideType) {
        let current = match side {
            AquaPathSideType::Right => self.right_controller.get_pid_ki(),
            AquaPathSideType::Left => self.left_controller.get_pid_ki(),
        };
        let value = Self::sanitize_clamped(value, Self::PID_MIN, Self::PID_MAX, current);
        match side {
            AquaPathSideType::Right => self.right_controller.set_pid_ki(value),
            AquaPathSideType::Left => self.left_controller.set_pid_ki(value),
        }
        self.emit_state();
    }

    fn set_pid_kd(&mut self, value: f64, side: AquaPathSideType) {
        let current = match side {
            AquaPathSideType::Right => self.right_controller.get_pid_kd(),
            AquaPathSideType::Left => self.left_controller.get_pid_kd(),
        };
        let value = Self::sanitize_clamped(value, Self::PID_MIN, Self::PID_MAX, current);
        match side {
            AquaPathSideType::Right => self.right_controller.set_pid_kd(value),
            AquaPathSideType::Left => self.left_controller.set_pid_kd(value),
        }
        self.emit_state();
    }

    fn set_thermal_flow_settle_duration(&mut self, duration: f64, side: AquaPathSideType) {
        if !self.mode_allows_standby_only_config() {
            return;
        }

        let current = match side {
            AquaPathSideType::Right => self
                .right_controller
                .get_thermal_flow_settle_duration()
                .as_secs_f64(),
            AquaPathSideType::Left => self
                .left_controller
                .get_thermal_flow_settle_duration()
                .as_secs_f64(),
        };
        let value = Self::sanitize_clamped(
            duration,
            Self::THERMAL_FLOW_SETTLE_DURATION_MIN,
            Self::THERMAL_FLOW_SETTLE_DURATION_MAX,
            current,
        );
        let duration = std::time::Duration::from_secs_f64(value);

        match side {
            AquaPathSideType::Right => self
                .right_controller
                .set_thermal_flow_settle_duration(duration),
            AquaPathSideType::Left => self
                .left_controller
                .set_thermal_flow_settle_duration(duration),
        }
        self.emit_state();
    }

    fn set_pump_cooldown_min_temperature(&mut self, temperature: f64, side: AquaPathSideType) {
        if !self.mode_allows_standby_only_config() {
            return;
        }

        let current = match side {
            AquaPathSideType::Right => self
                .right_controller
                .get_pump_cooldown_min_temperature()
                .get::<degree_celsius>(),
            AquaPathSideType::Left => self
                .left_controller
                .get_pump_cooldown_min_temperature()
                .get::<degree_celsius>(),
        };
        let value = Self::sanitize_clamped(
            temperature,
            Self::PUMP_COOLDOWN_MIN_TEMPERATURE_MIN,
            Self::PUMP_COOLDOWN_MIN_TEMPERATURE_MAX,
            current,
        );
        let temperature = ThermodynamicTemperature::new::<degree_celsius>(value);

        match side {
            AquaPathSideType::Right => self
                .right_controller
                .set_pump_cooldown_min_temperature(temperature),
            AquaPathSideType::Left => self
                .left_controller
                .set_pump_cooldown_min_temperature(temperature),
        }
        self.emit_state();
    }
}
