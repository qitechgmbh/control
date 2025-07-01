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
    TraverseSetLimitOuter(f64),
    /// Position in mm from home point
    TraverseSetLimitInner(f64),
    /// Step size in mm for traverse movement
    TraverseSetStepSize(f64),
    /// Padding in mm for traverse movement limits
    TraverseSetPadding(f64),
    TraverseGotoLimitOuter,
    TraverseGotoLimitInner,
    /// Find home point
    TraverseGotoHome,
    TraverseEnableLaserpointer(bool),

    // Puller
    /// on = speed, off = stop
    PullerSetRegulationMode(PullerRegulationMode),
    PullerSetTargetSpeed(f64),
    PullerSetTargetDiameter(f64),
    PullerSetForward(bool),

    // Spool Speed Controller
    SpoolSetRegulationMode(super::spool_speed_controller::SpoolSpeedControllerType),
    SpoolSetMinMaxMinSpeed(f64),
    SpoolSetMinMaxMaxSpeed(f64),

    // Adaptive Spool Speed Controller Parameters
    SpoolSetAdaptiveTensionTarget(f64),
    SpoolSetAdaptiveRadiusLearningRate(f64),
    SpoolSetAdaptiveMaxSpeedMultiplier(f64),
    SpoolSetAdaptiveAccelerationFactor(f64),
    SpoolSetAdaptiveDeaccelerationUrgencyMultiplier(f64),

    // Tension Arm
    TensionArmAngleZero,

    // Mode
    ModeSet(Mode),
}

#[derive(Serialize, Debug, Clone)]
pub struct TraversePositionEvent {
    /// position in mm
    pub position: Option<f64>,
}

impl TraversePositionEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("TraversePositionEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TraverseStateEvent {
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

impl TraverseStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("TraverseStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PullerStateEvent {
    /// regulation type
    pub regulation: PullerRegulationMode,
    /// target speed in m/min
    pub target_speed: f64,
    /// target diameter in mm
    pub target_diameter: f64,
    /// forward rotation direction
    pub forward: bool,
}

impl PullerStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("PullerStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PullerSpeedEvent {
    /// speed in m/min
    pub speed: f64,
}

impl PullerSpeedEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("PullerSpeedEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeStateEvent {
    /// mode
    pub mode: Mode,
    /// can wind
    pub can_wind: bool,
}

impl ModeStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("ModeStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SpoolRpmEvent {
    /// rpm
    pub rpm: f64,
}

impl SpoolRpmEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("SpoolRpmEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SpoolDiameterEvent {
    /// diameter in mm
    pub diameter: f64,
}

impl SpoolDiameterEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("SpoolDiameterEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct TensionArmAngleEvent {
    /// degree
    pub degree: f64,
}

impl TensionArmAngleEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("TensionArmAngleEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct TensionArmStateEvent {
    /// degree
    pub zeroed: bool,
}

impl TensionArmStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("TensionArmStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SpoolSpeedControllerStateEvent {
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

impl SpoolSpeedControllerStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("SpoolSpeedControllerStateEvent", self.clone())
    }
}

pub enum Winder2Events {
    TraversePosition(Event<TraversePositionEvent>),
    TraverseState(Event<TraverseStateEvent>),
    PullerSpeed(Event<PullerSpeedEvent>),
    PullerState(Event<PullerStateEvent>),
    Mode(Event<ModeStateEvent>),
    SpoolRpm(Event<SpoolRpmEvent>),
    SpoolDiameter(Event<SpoolDiameterEvent>),
    TensionArmAngleEvent(Event<TensionArmAngleEvent>),
    TensionArmStateEvent(Event<TensionArmStateEvent>),
    SpoolSpeedControllerStateEvent(Event<SpoolSpeedControllerStateEvent>),
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
            Winder2Events::TraversePosition(event) => event.into(),
            Winder2Events::TraverseState(event) => event.into(),
            Winder2Events::PullerSpeed(event) => event.into(),
            Winder2Events::PullerState(event) => event.into(),
            Winder2Events::Mode(event) => event.into(),
            Winder2Events::SpoolRpm(event) => event.into(),
            Winder2Events::SpoolDiameter(event) => event.into(),
            Winder2Events::TensionArmAngleEvent(event) => event.into(),
            Winder2Events::TensionArmStateEvent(event) => event.into(),
            Winder2Events::SpoolSpeedControllerStateEvent(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            Winder2Events::TraversePosition(_) => cache_one_hour,
            Winder2Events::TraverseState(_) => cache_one,
            Winder2Events::PullerSpeed(_) => cache_one_hour,
            Winder2Events::PullerState(_) => cache_one,
            Winder2Events::Mode(_) => cache_one,
            Winder2Events::SpoolRpm(_) => cache_one_hour,
            Winder2Events::SpoolDiameter(_) => cache_one_hour,
            Winder2Events::TensionArmAngleEvent(_) => cache_one_hour,
            Winder2Events::TensionArmStateEvent(_) => cache_one,
            Winder2Events::SpoolSpeedControllerStateEvent(_) => cache_one,
        }
    }
}

impl MachineApi for Winder2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::TraverseEnableLaserpointer(enable) => self.set_laser(enable),
            Mutation::ModeSet(mode) => self.set_mode(&mode.into()),
            Mutation::TraverseSetLimitOuter(limit) => self.traverse_set_limit_outer(limit),
            Mutation::TraverseSetLimitInner(limit) => self.traverse_set_limit_inner(limit),
            Mutation::TraverseSetStepSize(size) => self.traverse_set_step_size(size),
            Mutation::TraverseSetPadding(padding) => self.traverse_set_padding(padding),
            Mutation::TraverseGotoLimitOuter => self.traverse_goto_limit_outer(),
            Mutation::TraverseGotoLimitInner => self.traverse_goto_limit_inner(),
            Mutation::TraverseGotoHome => self.traverse_goto_home(),
            Mutation::PullerSetRegulationMode(regulation) => self.puller_set_regulation(regulation),
            Mutation::PullerSetTargetSpeed(value) => self.puller_set_target_speed(value),
            Mutation::PullerSetTargetDiameter(_) => todo!(),
            Mutation::PullerSetForward(value) => self.puller_set_forward(value),
            Mutation::SpoolSetRegulationMode(mode) => self.spool_set_regulation_mode(mode),
            Mutation::SpoolSetMinMaxMinSpeed(speed) => self.spool_set_minmax_min_speed(speed),
            Mutation::SpoolSetMinMaxMaxSpeed(speed) => self.spool_set_minmax_max_speed(speed),
            Mutation::SpoolSetAdaptiveTensionTarget(value) => {
                self.spool_set_adaptive_tension_target(value)
            }
            Mutation::SpoolSetAdaptiveRadiusLearningRate(value) => {
                self.spool_set_adaptive_radius_learning_rate(value)
            }
            Mutation::SpoolSetAdaptiveMaxSpeedMultiplier(value) => {
                self.spool_set_adaptive_max_speed_multiplier(value)
            }
            Mutation::SpoolSetAdaptiveAccelerationFactor(value) => {
                self.spool_set_adaptive_acceleration_factor(value)
            }
            Mutation::SpoolSetAdaptiveDeaccelerationUrgencyMultiplier(value) => {
                self.spool_set_adaptive_deacceleration_urgency_multiplier(value)
            }
            Mutation::TensionArmAngleZero => self.tension_arm_zero(),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.namespace
    }
}
