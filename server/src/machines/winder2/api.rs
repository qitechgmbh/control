use std::{sync::Arc, time::Duration};

use super::{Winder2, Winder2Mode, puller_speed_controller::PullerRegulationMode};
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_duration,
            cache_one_event,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Mode {
    Standby,
    Hold,
    Pull,
    Wind,
}

impl From<Winder2Mode> for Mode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Mode::Standby,
            Winder2Mode::Hold => Mode::Hold,
            Winder2Mode::Pull => Mode::Pull,
            Winder2Mode::Wind => Mode::Wind,
        }
    }
}

impl From<Mode> for Winder2Mode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Standby => Winder2Mode::Standby,
            Mode::Hold => Winder2Mode::Hold,
            Mode::Pull => Winder2Mode::Pull,
            Mode::Wind => Winder2Mode::Wind,
        }
    }
}

#[derive(Deserialize, Serialize)]
enum Mutation {
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

    // Spool Speed Controller
    SetSpoolRegulationMode(super::spool_speed_controller::SpoolSpeedControllerType),
    SetSpoolMinMaxMinSpeed(f64),
    SetSpoolMinMaxMaxSpeed(f64),

    // Adaptive Spool Speed Controller Parameters
    SetSpoolAdaptiveTensionTarget(f64),
    SetSpoolAdaptiveRadiusLearningRate(f64),
    SetSpoolAdaptiveMaxSpeedMultiplier(f64),
    SetSpoolAdaptiveAccelerationFactor(f64),
    SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(f64),

    // Tension Arm
    ZeroTensionArmAngle,

    // Mode
    SetMode(Mode),
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    /// traverse position in mm
    pub traverse_position: Option<f64>,
    /// puller speed in m/min
    pub puller_speed: f64,
    /// spool rpm
    pub spool_rpm: f64,
    /// spool diameter in mm
    pub spool_diameter: f64,
    /// tension arm angle in degrees
    pub tension_arm_angle: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    /// traverse state
    pub traverse_state: TraverseState,
    /// puller state
    pub puller_state: PullerState,
    /// mode state
    pub mode_state: ModeState,
    /// tension arm state
    pub tension_arm_state: TensionArmState,
    /// spool speed controller state
    pub spool_speed_controller_state: SpoolSpeedControllerState,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
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

#[derive(Serialize, Debug, Clone)]
pub struct PullerState {
    /// regulation type
    pub regulation: PullerRegulationMode,
    /// target speed in m/min
    pub target_speed: f64,
    /// target diameter in mm
    pub target_diameter: f64,
    /// forward rotation direction
    pub forward: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeState {
    /// mode
    pub mode: Mode,
    /// can wind
    pub can_wind: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct TensionArmState {
    /// is zeroed
    pub zeroed: bool,
}

#[derive(Serialize, Debug, Clone)]
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
}

pub enum Winder2Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct Winder2Namespace {
    pub namespace: Namespace,
}

impl NamespaceCacheingLogic<Winder2Events> for Winder2Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: Winder2Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        self.namespace.emit(event, &buffer_fn);
    }
}

impl Winder2Namespace {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
        }
    }
}

impl CacheableEvents<Winder2Events> for Winder2Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            Winder2Events::LiveValues(event) => event.into(),
            Winder2Events::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            Winder2Events::LiveValues(_) => cache_one_hour,
            Winder2Events::State(_) => cache_one,
        }
    }
}

impl MachineApi for Winder2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
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
            Mutation::SetSpoolRegulationMode(mode) => self.spool_set_regulation_mode(mode),
            Mutation::SetSpoolMinMaxMinSpeed(speed) => self.spool_set_minmax_min_speed(speed),
            Mutation::SetSpoolMinMaxMaxSpeed(speed) => self.spool_set_minmax_max_speed(speed),
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
            Mutation::ZeroTensionArmAngle => self.tension_arm_zero(),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.namespace
    }
}
