use api::{ToleranceState, ToleranceStates};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use units::angular_velocity::revolution_per_minute;
use units::f64::*;
use units::{thermodynamic_temperature::degree_celsius, volume_rate::liter_per_minute};

use crate::{AsyncThreadMessage, Machine, MachineMessage};
use crate::{
    MACHINE_AQUAPATH_V1, VENDOR_QITECH,
    aquapath1::{
        api::{
            AquaPathV1Events, AquaPathV1Namespace, CoolingModeState, CoolingModeStates, FanState,
            FanStates, FlowState, FlowStates, LiveValuesEvent, ModeState, NoticeEvent, PidState,
            PidStates, StateEvent, TempState, TempStates, ThermalSafetyState, ThermalSafetyStates,
        },
        controller::{ControlResetReason, Controller, ControllerNotice},
    },
    machine_identification::MachineIdentification,
};

use super::machine_identification::MachineIdentificationUnique;
use smol::channel::{Receiver, Sender};

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
    Front,
    Back,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Temperature {
    pub temperature: ThermodynamicTemperature,
    pub cooling: bool,
    pub heating: bool,
    pub target_temperature: ThermodynamicTemperature,
}

impl Machine for AquaPathV1 {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
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

#[derive(Debug)]
pub struct AquaPathV1 {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: AquaPathV1Namespace,
    mode: AquaPathV1Mode,
    ambient_temperature_calibration: ThermodynamicTemperature,
    last_measurement_emit: Instant,
    front_controller: Controller,
    back_controller: Controller,
    main_sender: Option<Sender<AsyncThreadMessage>>,
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
            front_temperature: self
                .front_controller
                .current_temperature
                .get::<degree_celsius>(),
            back_temperature: self
                .back_controller
                .current_temperature
                .get::<degree_celsius>(),
            front_flow: self.front_controller.current_flow.get::<liter_per_minute>(),
            back_flow: self.back_controller.current_flow.get::<liter_per_minute>(),
            front_temp_reservoir: self.front_controller.temp_reservoir.get::<degree_celsius>(),
            back_temp_reservoir: self.back_controller.temp_reservoir.get::<degree_celsius>(),
            front_revolutions: self
                .front_controller
                .current_revolutions
                .get::<revolution_per_minute>(),
            back_revolutions: self
                .back_controller
                .current_revolutions
                .get::<revolution_per_minute>(),
            front_power: self.front_controller.get_current_power(),
            back_power: self.back_controller.get_current_power(),
            front_heating: self.front_controller.temperature.heating,
            back_heating: self.back_controller.temperature.heating,
            front_cooling_mode: self.front_controller.cooling_mode,
            back_cooling_mode: self.back_controller.cooling_mode,
            front_pump_cooldown_active: self.front_controller.is_pump_cooldown_active(now),
            back_pump_cooldown_active: self.back_controller.is_pump_cooldown_active(now),
            front_pump_cooldown_remaining: self
                .front_controller
                .get_pump_cooldown_remaining(now)
                .as_secs_f64(),
            back_pump_cooldown_remaining: self
                .back_controller
                .get_pump_cooldown_remaining(now)
                .as_secs_f64(),
            front_heating_startup_wait_active: self
                .front_controller
                .is_heating_startup_wait_active(now),
            back_heating_startup_wait_active: self
                .back_controller
                .is_heating_startup_wait_active(now),
            front_heating_startup_wait_remaining: self
                .front_controller
                .get_heating_startup_wait_remaining(now)
                .as_secs_f64(),
            back_heating_startup_wait_remaining: self
                .back_controller
                .get_heating_startup_wait_remaining(now)
                .as_secs_f64(),
            front_total_energy: self.front_controller.get_total_energy(),
            back_total_energy: self.back_controller.get_total_energy(),
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
                front: TempState {
                    temperature: self
                        .front_controller
                        .current_temperature
                        .get::<degree_celsius>(),

                    target_temperature: self
                        .front_controller
                        .target_temperature
                        .get::<degree_celsius>(),
                },
                back: TempState {
                    temperature: self
                        .back_controller
                        .current_temperature
                        .get::<degree_celsius>(),
                    target_temperature: self
                        .back_controller
                        .target_temperature
                        .get::<degree_celsius>(),
                },
            },
            flow_states: FlowStates {
                front: FlowState {
                    flow: self.front_controller.current_flow.get::<liter_per_minute>(),
                    should_flow: self.front_controller.should_pump,
                },
                back: FlowState {
                    flow: self.back_controller.current_flow.get::<liter_per_minute>(),
                    should_flow: self.back_controller.should_pump,
                },
            },
            fan_states: FanStates {
                front: FanState {
                    revolutions: self
                        .front_controller
                        .max_revolutions
                        .get::<revolution_per_minute>(),
                    max_revolutions: self
                        .front_controller
                        .max_revolutions
                        .get::<revolution_per_minute>(),
                },
                back: FanState {
                    revolutions: self
                        .back_controller
                        .max_revolutions
                        .get::<revolution_per_minute>(),
                    max_revolutions: self
                        .back_controller
                        .max_revolutions
                        .get::<revolution_per_minute>(),
                },
            },
            cooling_mode_states: CoolingModeStates {
                front: CoolingModeState {
                    mode: self.front_controller.cooling_mode,
                },
                back: CoolingModeState {
                    mode: self.back_controller.cooling_mode,
                },
            },
            tolerance_states: ToleranceStates {
                front: ToleranceState {
                    heating: self
                        .front_controller
                        .heating_tolerance
                        .get::<degree_celsius>(),
                    cooling: self
                        .front_controller
                        .cooling_tolerance
                        .get::<degree_celsius>(),
                },
                back: ToleranceState {
                    heating: self
                        .back_controller
                        .heating_tolerance
                        .get::<degree_celsius>(),
                    cooling: self
                        .back_controller
                        .cooling_tolerance
                        .get::<degree_celsius>(),
                },
            },
            pid_states: PidStates {
                front: PidState {
                    kp: self.front_controller.get_pid_kp(),
                    ki: self.front_controller.get_pid_ki(),
                    kd: self.front_controller.get_pid_kd(),
                },
                back: PidState {
                    kp: self.back_controller.get_pid_kp(),
                    ki: self.back_controller.get_pid_ki(),
                    kd: self.back_controller.get_pid_kd(),
                },
            },
            thermal_safety_states: ThermalSafetyStates {
                front: ThermalSafetyState {
                    thermal_delay: self
                        .front_controller
                        .get_thermal_flow_settle_duration()
                        .as_secs_f64(),
                    cooldown_min_temperature: self
                        .front_controller
                        .get_pump_cooldown_min_temperature()
                        .get::<degree_celsius>(),
                },
                back: ThermalSafetyState {
                    thermal_delay: self
                        .back_controller
                        .get_thermal_flow_settle_duration()
                        .as_secs_f64(),
                    cooldown_min_temperature: self
                        .back_controller
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
        self.front_controller.set_should_pump(false);
        self.back_controller.set_should_pump(false);
    }

    fn turn_cooling_off(&mut self) {
        self.front_controller.disable_cooling();
        self.back_controller.disable_cooling();
    }

    fn turn_cooling_on(&mut self) {
        self.front_controller.allow_cooling();
        self.back_controller.allow_cooling();
    }

    fn turn_heating_off(&mut self) {
        self.front_controller.disallow_heating();
        self.back_controller.disallow_heating();
    }

    fn turn_heating_on(&mut self) {
        self.front_controller.allow_heating();
        self.back_controller.allow_heating();
    }

    fn turn_pump_on(&mut self) {
        self.front_controller.allow_pump();
        self.back_controller.allow_pump();
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
        let min = self
            .front_controller
            .min_temperature
            .get::<degree_celsius>();
        let max = self
            .front_controller
            .max_temperature
            .get::<degree_celsius>();
        ThermodynamicTemperature::new::<degree_celsius>(ambient.max(min).min(max))
    }

    fn set_ambient_temperature_calibration(&mut self, ambient_temperature: f64) {
        let clamped = ambient_temperature
            .max(
                self.front_controller
                    .min_temperature
                    .get::<degree_celsius>(),
            )
            .min(
                self.front_controller
                    .max_temperature
                    .get::<degree_celsius>(),
            );
        self.ambient_temperature_calibration =
            ThermodynamicTemperature::new::<degree_celsius>(clamped);

        // Enforce the calibrated minimum immediately on already-configured targets.
        let min_settable = self.get_min_settable_temperature();
        let min_settable_temperature = min_settable.get::<degree_celsius>();

        if self
            .front_controller
            .target_temperature
            .get::<degree_celsius>()
            < min_settable_temperature
        {
            self.front_controller.set_target_temperature(min_settable);
        }
        if self
            .back_controller
            .target_temperature
            .get::<degree_celsius>()
            < min_settable_temperature
        {
            self.back_controller.set_target_temperature(min_settable);
        }

        self.emit_state();
    }

    fn set_target_temperature(&mut self, temperature: f64, cooling_type: AquaPathSideType) {
        let min_settable = self.get_min_settable_temperature().get::<degree_celsius>();
        let max_settable = self
            .front_controller
            .max_temperature
            .get::<degree_celsius>();
        let clamped_target = temperature.max(min_settable).min(max_settable);
        let target_temp = ThermodynamicTemperature::new::<degree_celsius>(clamped_target);

        match cooling_type {
            AquaPathSideType::Back => self.back_controller.set_target_temperature(target_temp),
            AquaPathSideType::Front => self.front_controller.set_target_temperature(target_temp),
        }
        self.emit_state();
    }

    fn set_should_pump(&mut self, should_pump: bool, cooling_type: AquaPathSideType) {
        match cooling_type {
            AquaPathSideType::Back => self.back_controller.set_should_pump(should_pump),
            AquaPathSideType::Front => self.front_controller.set_should_pump(should_pump),
        }
        self.emit_state();
    }
}

impl AquaPathV1 {
    fn set_max_revolutions(&mut self, revolutions: f64, fan_type: AquaPathSideType) {
        match fan_type {
            AquaPathSideType::Back => self
                .back_controller
                .set_max_revolutions(AngularVelocity::new::<revolution_per_minute>(revolutions)),
            AquaPathSideType::Front => self
                .front_controller
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
            AquaPathSideType::Back => {
                let fallback = self
                    .back_controller
                    .heating_tolerance
                    .get::<degree_celsius>();
                let value = Self::sanitize_clamped(
                    tolerance,
                    Self::TOLERANCE_MIN,
                    Self::TOLERANCE_MAX,
                    fallback,
                );
                self.back_controller
                    .set_heating_tolerance(ThermodynamicTemperature::new::<degree_celsius>(value))
            }
            AquaPathSideType::Front => {
                let fallback = self
                    .front_controller
                    .heating_tolerance
                    .get::<degree_celsius>();
                let value = Self::sanitize_clamped(
                    tolerance,
                    Self::TOLERANCE_MIN,
                    Self::TOLERANCE_MAX,
                    fallback,
                );
                self.front_controller
                    .set_heating_tolerance(ThermodynamicTemperature::new::<degree_celsius>(value))
            }
        }

        self.emit_state();
    }

    fn set_cooling_tolerance(&mut self, tolerance: f64, tolerance_type: AquaPathSideType) {
        match tolerance_type {
            AquaPathSideType::Back => {
                let fallback = self
                    .back_controller
                    .cooling_tolerance
                    .get::<degree_celsius>();
                let value = Self::sanitize_clamped(
                    tolerance,
                    Self::TOLERANCE_MIN,
                    Self::TOLERANCE_MAX,
                    fallback,
                );
                self.back_controller
                    .set_cooling_tolerance(ThermodynamicTemperature::new::<degree_celsius>(value))
            }
            AquaPathSideType::Front => {
                let fallback = self
                    .front_controller
                    .cooling_tolerance
                    .get::<degree_celsius>();
                let value = Self::sanitize_clamped(
                    tolerance,
                    Self::TOLERANCE_MIN,
                    Self::TOLERANCE_MAX,
                    fallback,
                );
                self.front_controller
                    .set_cooling_tolerance(ThermodynamicTemperature::new::<degree_celsius>(value))
            }
        }

        self.emit_state();
    }

    fn set_pid_kp(&mut self, value: f64, side: AquaPathSideType) {
        let current = match side {
            AquaPathSideType::Back => self.back_controller.get_pid_kp(),
            AquaPathSideType::Front => self.front_controller.get_pid_kp(),
        };
        let value = Self::sanitize_clamped(value, Self::PID_MIN, Self::PID_MAX, current);
        match side {
            AquaPathSideType::Back => self.back_controller.set_pid_kp(value),
            AquaPathSideType::Front => self.front_controller.set_pid_kp(value),
        }
        self.emit_state();
    }

    fn set_pid_ki(&mut self, value: f64, side: AquaPathSideType) {
        let current = match side {
            AquaPathSideType::Back => self.back_controller.get_pid_ki(),
            AquaPathSideType::Front => self.front_controller.get_pid_ki(),
        };
        let value = Self::sanitize_clamped(value, Self::PID_MIN, Self::PID_MAX, current);
        match side {
            AquaPathSideType::Back => self.back_controller.set_pid_ki(value),
            AquaPathSideType::Front => self.front_controller.set_pid_ki(value),
        }
        self.emit_state();
    }

    fn set_pid_kd(&mut self, value: f64, side: AquaPathSideType) {
        let current = match side {
            AquaPathSideType::Back => self.back_controller.get_pid_kd(),
            AquaPathSideType::Front => self.front_controller.get_pid_kd(),
        };
        let value = Self::sanitize_clamped(value, Self::PID_MIN, Self::PID_MAX, current);
        match side {
            AquaPathSideType::Back => self.back_controller.set_pid_kd(value),
            AquaPathSideType::Front => self.front_controller.set_pid_kd(value),
        }
        self.emit_state();
    }

    fn set_thermal_flow_settle_duration(&mut self, duration: f64, side: AquaPathSideType) {
        if !self.mode_allows_standby_only_config() {
            return;
        }

        let current = match side {
            AquaPathSideType::Back => self
                .back_controller
                .get_thermal_flow_settle_duration()
                .as_secs_f64(),
            AquaPathSideType::Front => self
                .front_controller
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
            AquaPathSideType::Back => self
                .back_controller
                .set_thermal_flow_settle_duration(duration),
            AquaPathSideType::Front => self
                .front_controller
                .set_thermal_flow_settle_duration(duration),
        }
        self.emit_state();
    }

    fn set_pump_cooldown_min_temperature(&mut self, temperature: f64, side: AquaPathSideType) {
        if !self.mode_allows_standby_only_config() {
            return;
        }

        let current = match side {
            AquaPathSideType::Back => self
                .back_controller
                .get_pump_cooldown_min_temperature()
                .get::<degree_celsius>(),
            AquaPathSideType::Front => self
                .front_controller
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
            AquaPathSideType::Back => self
                .back_controller
                .set_pump_cooldown_min_temperature(temperature),
            AquaPathSideType::Front => self
                .front_controller
                .set_pump_cooldown_min_temperature(temperature),
        }
        self.emit_state();
    }
}
