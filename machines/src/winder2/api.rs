#[cfg(feature = "mock-machine")]
mod winder2_imports {
    pub use super::super::Winder2Mode;
    pub use super::super::puller_speed_controller::{GearRatio, PullerRegulationMode};
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
}

#[cfg(not(feature = "mock-machine"))]
mod winder2_imports {
    pub use super::super::puller_speed_controller::{GearRatio, PullerRegulationMode};
    pub use super::super::{Winder2, Winder2Mode};
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
}

#[cfg(not(feature = "mock-machine"))]
use smol::channel::Sender;
pub use winder2_imports::*;

#[cfg(not(feature = "mock-machine"))]
use crate::{MachineApi, MachineMessage};
use crate::{MachineCrossConnectionState, machine_identification::MachineIdentificationUnique};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}

impl From<Winder2Mode> for Mode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Self::Standby,
            Winder2Mode::Hold => Self::Hold,
            Winder2Mode::Pull => Self::Pull,
            Winder2Mode::Wind => Self::Wind,
        }
    }
}

impl From<Mode> for Winder2Mode {
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
    SetSpoolRegulationMode(super::spool_speed_controller::SpoolSpeedControllerType),
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

    // Mode
    SetMode(Mode),

    // Connected Machine
    SetConnectedMachine(MachineIdentificationUnique),

    // Disconnect Machine
    DisconnectMachine(MachineIdentificationUnique),
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
    /// Is a Machine Connected?
    pub connected_machine_state: MachineCrossConnectionState,
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
    pub regulation_mode: super::spool_speed_controller::SpoolSpeedControllerType,
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

pub enum Winder2Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct Winder2Namespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<Winder2Events> for Winder2Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: Winder2Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}

impl CacheableEvents<Self> for Winder2Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_first_and_last = cache_first_and_last_event();
        match self {
            Self::LiveValues(_) => cache_first_and_last,
            Self::State(_) => cache_first_and_last,
        }
    }
}

#[cfg(not(feature = "mock-machine"))]
impl MachineApi for Winder2 {
    fn api_get_sender(&self) -> Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        use crate::Machine;

        let mutation: Mutation = serde_json::from_value(request_body)?;
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
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
