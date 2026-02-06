mod gluetex_imports {
    pub use super::super::controllers::puller_speed_controller::{GearRatio, PullerRegulationMode};
    pub use super::super::{Gluetex, GluetexMode};
    pub use control_core::socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_duration,
            cache_first_and_last_event,
        },
    };

    pub use control_core_derive::BuildEvent;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::Value;
    pub use smol::lock::Mutex;
    pub use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    pub use tracing::instrument;
    pub use units::{Angle, angle::degree};
}

pub use gluetex_imports::*;
use smol::channel::Sender;

use crate::{MachineApi, MachineMessage};
use crate::{MachineCrossConnectionState, machine_identification::MachineIdentificationUnique};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum HeatingZone {
    Zone1,
    Zone2,
    Zone3,
    Zone4,
    Zone5,
    Zone6,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct HeatingPidSettings {
    pub ki: f64,
    pub kp: f64,
    pub kd: f64,
    pub zone: String,
}

#[derive(Serialize, Debug, Clone, BuildEvent)]
pub struct HeatingAutoTuneCompleteEvent {
    pub zone: String,
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}

impl From<GluetexMode> for Mode {
    fn from(mode: GluetexMode) -> Self {
        match mode {
            GluetexMode::Standby => Self::Standby,
            GluetexMode::Hold => Self::Hold,
            GluetexMode::Pull => Self::Pull,
            GluetexMode::Wind => Self::Wind,
        }
    }
}

impl From<Mode> for GluetexMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Standby => Self::Standby,
            Mode::Hold => Self::Hold,
            Mode::Pull => Self::Pull,
            Mode::Wind => Self::Wind,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum Mutation {
    // Traverse
    /// Position in mm from home point
    SetTraverseLimitOuter(f64),
    /// Position in mm from home point
    SetTraverseLimitInner(f64),
    /// Step size in mm for traverse movement
    SetTraverseStepSize(f64),
    /// Padding in mm for traverse movement limits
    SetTraversePadding(f64),
    GotoTraverseLimitOuter,
    GotoTraverseLimitInner,
    /// Find home point
    GotoTraverseHome,
    EnableTraverseLaserpointer(bool),

    // Puller
    /// on = speed, off = stop
    SetPullerRegulationMode(PullerRegulationMode),
    SetPullerTargetSpeed(f64),
    SetPullerTargetDiameter(f64),
    SetPullerForward(bool),
    SetPullerGearRatio(GearRatio),

    // Spool Speed Controller
    SetSpoolRegulationMode(super::controllers::spool_speed_controller::SpoolSpeedControllerType),
    SetSpoolMinMaxMinSpeed(f64),
    SetSpoolMinMaxMaxSpeed(f64),
    SetSpoolForward(bool),

    // Adaptive Spool Speed Controller Parameters
    SetSpoolAdaptiveTensionTarget(f64),
    SetSpoolAdaptiveRadiusLearningRate(f64),
    SetSpoolAdaptiveMaxSpeedMultiplier(f64),
    SetSpoolAdaptiveAccelerationFactor(f64),
    SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(f64),

    // Spool Auto Stop/Pull
    SetSpoolAutomaticRequiredMeters(f64),
    SetSpoolAutomaticAction(SpoolAutomaticActionMode),
    ResetSpoolProgress,

    // Tension Arm
    ZeroTensionArmAngle,

    // Heating
    SetHeatingEnabled(bool),
    SetHeatingTargetTemperature(HeatingZone, f64),
    ConfigureHeatingPid(HeatingPidSettings),
    StartHeatingAutoTune(HeatingZone, f64), // zone, target_temp
    StopHeatingAutoTune(HeatingZone),

    // Mode
    SetMode(Mode),

    // Connected Machine
    SetConnectedMachine(MachineIdentificationUnique),

    // Disconnect Machine
    DisconnectMachine(MachineIdentificationUnique),

    // Addon Motors
    SetAddonMotor3Enabled(bool),
    SetAddonMotor4Enabled(bool),
    SetAddonMotor5Enabled(bool),
    SetAddonMotor3Forward(bool),
    SetAddonMotor4Forward(bool),
    SetAddonMotor5Forward(bool),
    SetAddonMotor3MasterRatio(f64),
    SetAddonMotor3SlaveRatio(f64),
    SetAddonMotor4MasterRatio(f64),
    SetAddonMotor4SlaveRatio(f64),
    SetAddonMotor5MasterRatio(f64),
    SetAddonMotor5SlaveRatio(f64),
    SetAddonMotor3Konturlaenge(f64),
    SetAddonMotor3Pause(f64),

    // Slave Puller
    SetSlavePullerEnabled(bool),
    SetSlavePullerForward(bool),
    SetSlavePullerTargetAngle(f64),
    SetSlavePullerSensitivity(f64),
    SetSlavePullerMinSpeedFactor(f64),
    SetSlavePullerMaxSpeedFactor(f64),
    ZeroSlaveTensionArm,
    ZeroAddonTensionArm,

    // Tension Arm Monitoring
    SetTensionArmMonitorEnabled(bool),
    SetTensionArmMonitorMinAngle(f64),
    SetTensionArmMonitorMaxAngle(f64),

    // Sleep Timer
    SetSleepTimerEnabled(bool),
    SetSleepTimerTimeout(u64),
    ResetSleepTimer,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    /// traverse position in mm
    pub traverse_position: Option<f64>,
    /// puller speed in m/min
    pub puller_speed: f64,
    /// spool rpm
    pub spool_rpm: f64,
    /// tension arm angle in degrees
    pub tension_arm_angle: f64,
    // spool progress in meters (pulled distance of filament)
    pub spool_progress: f64,
    /// temperature 1 in celsius
    pub temperature_1: f64,
    /// temperature 2 in celsius
    pub temperature_2: f64,
    /// temperature 3 in celsius
    pub temperature_3: f64,
    /// temperature 4 in celsius
    pub temperature_4: f64,
    /// temperature 5 in celsius
    pub temperature_5: f64,
    /// temperature 6 in celsius
    pub temperature_6: f64,
    /// heater 1 power in watts
    pub heater_1_power: f64,
    /// heater 2 power in watts
    pub heater_2_power: f64,
    /// heater 3 power in watts
    pub heater_3_power: f64,
    /// heater 4 power in watts
    pub heater_4_power: f64,
    /// heater 5 power in watts
    pub heater_5_power: f64,
    /// heater 6 power in watts
    pub heater_6_power: f64,
    /// slave puller speed in m/min
    pub slave_puller_speed: f64,
    /// slave tension arm angle in degrees
    pub slave_tension_arm_angle: f64,
    /// addon tension arm angle in degrees
    pub addon_tension_arm_angle: f64,
    /// optris 1 voltage (role 9 AI2)
    pub optris_1_voltage: f64,
    /// optris 2 voltage (role 10 AI2)
    pub optris_2_voltage: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, BuildEvent)]
pub struct StateEvent {
    pub is_default_state: bool,
    /// traverse state
    pub traverse_state: TraverseState,
    /// puller state
    pub puller_state: PullerState,
    /// spool automatic action state and progress
    pub spool_automatic_action_state: SpoolAutomaticActionState,
    /// mode state
    pub mode_state: ModeState,
    /// tension arm state
    pub tension_arm_state: TensionArmState,
    /// spool speed controller state
    pub spool_speed_controller_state: SpoolSpeedControllerState,
    /// heating states
    pub heating_states: HeatingStates,
    /// PID settings for heating zones
    pub heating_pid_settings: HeatingPidStates,
    /// Is a Machine Connected?
    pub connected_machine_state: MachineCrossConnectionState,
    /// addon motor 3 state (konturrad with endstop)
    pub addon_motor_3_state: AddonMotor5State,
    /// addon motor 4 state
    pub addon_motor_4_state: AddonMotorState,
    /// addon motor 5 state
    pub addon_motor_5_state: AddonMotorState,
    /// slave puller state
    pub slave_puller_state: SlavePullerState,
    /// addon tension arm state
    pub addon_tension_arm_state: TensionArmState,
    /// tension arm monitor state
    pub tension_arm_monitor_state: TensionArmMonitorState,
    /// sleep timer state
    pub sleep_timer_state: SleepTimerState,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TraverseState {
    /// min position in mm
    pub limit_inner: f64,
    /// max position in mm
    pub limit_outer: f64,
    /// position in mm
    pub position_in: f64,
    /// position out in mm
    pub position_out: f64,
    /// is going to position in
    pub is_going_in: bool,
    /// is going to position out
    pub is_going_out: bool,
    /// if is homed
    pub is_homed: bool,
    /// if is homing
    pub is_going_home: bool,
    /// if is traversing
    pub is_traversing: bool,
    /// laserpointer is on
    pub laserpointer: bool,
    /// step size in mm
    pub step_size: f64,
    /// padding in mm
    pub padding: f64,
    /// can go in (to inner limit)
    pub can_go_in: bool,
    /// can go out (to outer limit)
    pub can_go_out: bool,
    /// can home
    pub can_go_home: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct PullerState {
    /// regulation type
    pub regulation: PullerRegulationMode,
    /// target speed in m/min
    pub target_speed: f64,
    /// target diameter in mm
    pub target_diameter: f64,
    /// forward rotation direction
    pub forward: bool,
    /// gear ratio for winding speed
    pub gear_ratio: GearRatio,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub enum SpoolAutomaticActionMode {
    #[default]
    NoAction,
    Pull,
    Hold,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct SpoolAutomaticActionState {
    pub spool_required_meters: f64,
    pub spool_automatic_action_mode: SpoolAutomaticActionMode,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct ModeState {
    /// mode
    pub mode: Mode,
    /// can wind
    pub can_wind: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TensionArmState {
    /// is zeroed
    pub zeroed: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct SpoolSpeedControllerState {
    /// regulation mode
    pub regulation_mode: super::controllers::spool_speed_controller::SpoolSpeedControllerType,
    /// min speed in rpm for minmax mode
    pub minmax_min_speed: f64,
    /// max speed in rpm for minmax mode
    pub minmax_max_speed: f64,
    /// tension target for adaptive mode (0.0-1.0)
    pub adaptive_tension_target: f64,
    /// radius learning rate for adaptive mode
    pub adaptive_radius_learning_rate: f64,
    /// max speed multiplier for adaptive mode
    pub adaptive_max_speed_multiplier: f64,
    /// acceleration factor for adaptive mode
    pub adaptive_acceleration_factor: f64,
    /// deacceleration urgency multiplier for adaptive mode
    pub adaptive_deacceleration_urgency_multiplier: f64,
    /// forward rotation direction
    pub forward: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct AddonMotorState {
    /// is motor enabled (running)
    pub enabled: bool,
    /// forward rotation direction
    pub forward: bool,
    /// master ratio value (e.g., 2 in "2:1")
    pub master_ratio: f64,
    /// slave ratio value (e.g., 1 in "2:1")
    pub slave_ratio: f64,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct AddonMotor5State {
    /// is motor enabled (running)
    pub enabled: bool,
    /// forward rotation direction
    pub forward: bool,
    /// master ratio value (e.g., 2 in "2:1")
    pub master_ratio: f64,
    /// slave ratio value (e.g., 1 in "2:1")
    pub slave_ratio: f64,
    /// Konturlänge in mm (0 = constant mode)
    pub konturlaenge_mm: f64,
    /// Pause in mm (0 = constant mode)
    pub pause_mm: f64,
    /// Current pattern control state
    pub pattern_state: String,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct SlavePullerState {
    /// is slave puller enabled
    pub enabled: bool,
    /// forward rotation direction
    pub forward: bool,
    /// target tension arm angle (setpoint in degrees)
    pub target_angle: f64,
    /// sensitivity range around target angle for speed adjustment (degrees)
    pub sensitivity: f64,
    /// minimum speed factor for overspeed protection (optional)
    pub min_speed_factor: Option<f64>,
    /// maximum speed factor for overspeed protection (optional)
    pub max_speed_factor: Option<f64>,
    /// slave tension arm state
    pub tension_arm: SlaveTensionArmState,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct SlaveTensionArmState {
    /// is zeroed
    pub zeroed: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct HeatingStates {
    pub enabled: bool,
    pub zone_1: HeatingState,
    pub zone_2: HeatingState,
    pub zone_3: HeatingState,
    pub zone_4: HeatingState,
    pub zone_5: HeatingState,
    pub zone_6: HeatingState,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct HeatingState {
    /// target temperature in celsius
    pub target_temperature: f64,
    /// wiring error detected
    pub wiring_error: bool,
    /// auto-tuning is active
    pub autotuning_active: bool,
    /// auto-tuning progress (0-100%)
    pub autotuning_progress: f64,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct HeatingPidStates {
    pub zone_1: HeatingPidSettings,
    pub zone_2: HeatingPidSettings,
    pub zone_3: HeatingPidSettings,
    pub zone_4: HeatingPidSettings,
    pub zone_5: HeatingPidSettings,
    pub zone_6: HeatingPidSettings,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TensionArmMonitorState {
    /// is monitoring enabled
    pub enabled: bool,
    /// minimum allowed angle in degrees
    pub min_angle: f64,
    /// maximum allowed angle in degrees
    pub max_angle: f64,
    /// is monitor currently triggered (limits exceeded)
    pub triggered: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct SleepTimerState {
    /// is sleep timer enabled
    pub enabled: bool,
    /// timeout in seconds
    pub timeout_seconds: u64,
    /// remaining seconds until sleep
    pub remaining_seconds: u64,
}

pub enum GluetexEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
    HeatingAutoTuneComplete(Event<HeatingAutoTuneCompleteEvent>),
}

#[derive(Debug)]
pub struct GluetexNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<GluetexEvents> for GluetexNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: GluetexEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}

impl CacheableEvents<Self> for GluetexEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
            Self::HeatingAutoTuneComplete(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_first_and_last = cache_first_and_last_event();
        match self {
            Self::LiveValues(_) => cache_first_and_last,
            Self::State(_) => cache_first_and_last,
            Self::HeatingAutoTuneComplete(_) => cache_first_and_last,
        }
    }
}

impl MachineApi for Gluetex {
    fn api_get_sender(&self) -> Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        use crate::Machine;

        let mutation: Mutation = serde_json::from_value(request_body)?;

        // Reset sleep timer on any user interaction (except explicit sleep timer commands)
        match mutation {
            Mutation::SetSleepTimerEnabled(_)
            | Mutation::SetSleepTimerTimeout(_)
            | Mutation::ResetSleepTimer => {
                // Don't reset timer for sleep timer configuration changes
            }
            _ => {
                // Reset timer for all other user interactions
                self.reset_sleep_timer();
            }
        }

        match mutation {
            Mutation::EnableTraverseLaserpointer(enable) => self.set_laser(enable),
            Mutation::SetMode(mode) => self.set_mode(&mode.into()),
            Mutation::SetTraverseLimitOuter(limit) => self.traverse_set_limit_outer(limit),
            Mutation::SetTraverseLimitInner(limit) => self.traverse_set_limit_inner(limit),
            Mutation::SetTraverseStepSize(size) => self.traverse_set_step_size(size),
            Mutation::SetTraversePadding(padding) => self.traverse_set_padding(padding),
            Mutation::GotoTraverseLimitOuter => self.traverse_goto_limit_outer(),
            Mutation::GotoTraverseLimitInner => self.traverse_goto_limit_inner(),
            Mutation::GotoTraverseHome => self.traverse_goto_home(),
            Mutation::SetPullerRegulationMode(regulation) => self.puller_set_regulation(regulation),
            Mutation::SetPullerTargetSpeed(value) => self.puller_set_target_speed(value),
            Mutation::SetPullerTargetDiameter(_) => todo!(),
            Mutation::SetPullerForward(value) => self.puller_set_forward(value),
            Mutation::SetPullerGearRatio(gear_ratio) => self.puller_set_gear_ratio(gear_ratio),
            Mutation::SetSpoolRegulationMode(mode) => self.spool_set_regulation_mode(mode),
            Mutation::SetSpoolMinMaxMinSpeed(speed) => self.spool_set_minmax_min_speed(speed),
            Mutation::SetSpoolMinMaxMaxSpeed(speed) => self.spool_set_minmax_max_speed(speed),
            Mutation::SetSpoolForward(value) => self.spool_set_forward(value),
            Mutation::SetSpoolAdaptiveTensionTarget(value) => {
                self.spool_set_adaptive_tension_target(value)
            }
            Mutation::SetSpoolAdaptiveRadiusLearningRate(value) => {
                self.spool_set_adaptive_radius_learning_rate(value)
            }
            Mutation::SetSpoolAdaptiveMaxSpeedMultiplier(value) => {
                self.spool_set_adaptive_max_speed_multiplier(value)
            }
            Mutation::SetSpoolAdaptiveAccelerationFactor(value) => {
                self.spool_set_adaptive_acceleration_factor(value)
            }
            Mutation::SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(value) => {
                self.spool_set_adaptive_deacceleration_urgency_multiplier(value)
            }
            Mutation::SetSpoolAutomaticRequiredMeters(meters) => {
                self.set_spool_automatic_required_meters(meters)
            }
            Mutation::SetSpoolAutomaticAction(mode) => self.set_spool_automatic_mode(mode),
            Mutation::ResetSpoolProgress => self.stop_or_pull_spool_reset(Instant::now()),
            Mutation::ZeroTensionArmAngle => self.tension_arm_zero(),
            Mutation::SetHeatingEnabled(enabled) => {
                self.set_heating_enabled(enabled);
                self.emit_state();
            }
            Mutation::SetHeatingTargetTemperature(zone, temperature) => {
                self.set_target_temperature(temperature, zone)
            }
            Mutation::ConfigureHeatingPid(settings) => self.configure_heating_pid(settings),
            Mutation::StartHeatingAutoTune(zone, target_temp) => {
                self.start_heating_autotune(zone, target_temp)
            }
            Mutation::StopHeatingAutoTune(zone) => self.stop_heating_autotune(zone),
            Mutation::SetConnectedMachine(machine_identification_unique) => {
                let main_sender = match &self.main_sender {
                    Some(sender) => sender,
                    None => {
                        return Err(anyhow::anyhow!(
                            "Machine cannot connect to others! {:?}",
                            self.get_machine_identification_unique()
                        ));
                    }
                };
                main_sender.try_send(crate::AsyncThreadMessage::ConnectOneWayRequest(
                    crate::CrossConnection {
                        src: self.get_machine_identification_unique(),
                        dest: machine_identification_unique,
                    },
                ))?;
                self.emit_state();
            }
            Mutation::DisconnectMachine(_machine_identification_unique) => {
                self.connected_machines.clear();
                /*let main_sender = match &self.main_sender {
                    Some(sender) => sender,
                    None => return Err(anyhow::anyhow!("[DisconnectMachine] Machine cannot connect to others! {:?}", self.machine_identification_unique)),
                };*/
                //main_sender.try_send(crate::AsyncThreadMessage::ConnectOneWayRequest(crate::CrossConnection { src: self.get_machine_identification_unique(), dest: machine_identification_unique }))?;
                self.emit_state();
            }
            Mutation::SetAddonMotor3Enabled(enabled) => {
                self.addon_motor_3_controller.set_enabled(enabled);
                self.emit_state();
            }
            Mutation::SetAddonMotor3Forward(forward) => {
                self.addon_motor_3_controller.set_forward(forward);
                self.emit_state();
            }
            Mutation::SetAddonMotor4Enabled(enabled) => {
                self.addon_motor_4_controller.set_enabled(enabled);
                self.emit_state();
            }
            Mutation::SetAddonMotor4Forward(forward) => {
                self.addon_motor_4_controller.set_forward(forward);
                self.emit_state();
            }
            Mutation::SetAddonMotor3MasterRatio(ratio) => {
                self.addon_motor_3_controller.set_master_ratio(ratio);
                self.emit_state();
            }
            Mutation::SetAddonMotor3SlaveRatio(ratio) => {
                self.addon_motor_3_controller.set_slave_ratio(ratio);
                self.emit_state();
            }
            Mutation::SetAddonMotor4MasterRatio(ratio) => {
                self.addon_motor_4_controller.set_master_ratio(ratio);
                self.emit_state();
            }
            Mutation::SetAddonMotor4SlaveRatio(ratio) => {
                self.addon_motor_4_controller.set_slave_ratio(ratio);
                self.emit_state();
            }
            Mutation::SetAddonMotor5Enabled(enabled) => {
                self.addon_motor_5_controller.set_enabled(enabled);
                self.emit_state();
            }
            Mutation::SetAddonMotor5Forward(forward) => {
                self.addon_motor_5_controller.set_forward(forward);
                self.emit_state();
            }
            Mutation::SetAddonMotor5MasterRatio(ratio) => {
                self.addon_motor_5_controller.set_master_ratio(ratio);
                self.emit_state();
            }
            Mutation::SetAddonMotor5SlaveRatio(ratio) => {
                self.addon_motor_5_controller.set_slave_ratio(ratio);
                self.emit_state();
            }
            Mutation::SetAddonMotor3Konturlaenge(length_mm) => {
                self.addon_motor_3_controller.set_konturlaenge_mm(length_mm);
                self.emit_state();
            }
            Mutation::SetAddonMotor3Pause(pause_mm) => {
                self.addon_motor_3_controller.set_pause_mm(pause_mm);
                self.emit_state();
            }
            Mutation::SetSlavePullerEnabled(enabled) => {
                self.slave_puller_user_enabled = enabled;
                // Re-apply the current mode to update the controller state
                // This ensures the mode transition logic respects the new user preference
                let current_mode = self.mode.clone();
                self.set_slave_puller_mode(&current_mode);
                self.emit_state();
            }
            Mutation::SetSlavePullerForward(forward) => {
                self.slave_puller_speed_controller.set_forward(forward);
                self.emit_state();
            }
            Mutation::SetSlavePullerTargetAngle(angle_deg) => {
                self.slave_puller_speed_controller
                    .set_target_angle(Angle::new::<degree>(angle_deg));
                self.emit_state();
            }
            Mutation::SetSlavePullerSensitivity(sensitivity_deg) => {
                self.slave_puller_speed_controller
                    .set_sensitivity(Angle::new::<degree>(sensitivity_deg));
                self.emit_state();
            }
            Mutation::SetSlavePullerMinSpeedFactor(factor) => {
                self.slave_puller_speed_controller
                    .set_min_speed_factor(Some(factor));
                self.emit_state();
            }
            Mutation::SetSlavePullerMaxSpeedFactor(factor) => {
                self.slave_puller_speed_controller
                    .set_max_speed_factor(Some(factor));
                self.emit_state();
            }
            Mutation::ZeroSlaveTensionArm => {
                self.slave_tension_arm.zero();
                self.emit_state();
            }
            Mutation::ZeroAddonTensionArm => {
                self.addon_tension_arm.zero();
                self.emit_state();
            }
            Mutation::SetTensionArmMonitorEnabled(enabled) => {
                self.tension_arm_monitor_config.enabled = enabled;
                // Clear triggered flag when disabling
                if !enabled {
                    self.tension_arm_monitor_triggered = false;
                }
                self.emit_state();
            }
            Mutation::SetTensionArmMonitorMinAngle(angle_deg) => {
                self.tension_arm_monitor_config.min_angle = Angle::new::<degree>(angle_deg);
                self.emit_state();
            }
            Mutation::SetTensionArmMonitorMaxAngle(angle_deg) => {
                self.tension_arm_monitor_config.max_angle = Angle::new::<degree>(angle_deg);
                self.emit_state();
            }
            Mutation::SetSleepTimerEnabled(enabled) => {
                self.sleep_timer_config.enabled = enabled;
                // Reset the timer when enabling
                if enabled {
                    self.reset_sleep_timer();
                }
                self.emit_state();
            }
            Mutation::SetSleepTimerTimeout(timeout_seconds) => {
                self.sleep_timer_config.timeout_seconds = timeout_seconds;
                self.emit_state();
            }
            Mutation::ResetSleepTimer => {
                self.reset_sleep_timer();
                self.emit_state();
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
