use std::time::Duration;

use super::{Winder2, Winder2Mode, puller_speed_controller::PullerRegulationMode};
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, NamespaceInterface,
            cache_duration, cache_one_event,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AutostopTransition {
    Standby,
    Pull,
}

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

    // Auto Stop
    AutostopEnable(bool),
    AutostopEnableAlarm(bool),
    AutostopSetLimit(f64),
    AutostopSetTransition(AutostopTransition),

    // Tension Arm
    TensionArmAngleZero,

    // Spool
    SpoolSetSpeedMax(f64),
    SpoolSetSpeedMin(f64),

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
pub struct AutostopWoundedLengthEvent {
    /// wounded length in mm
    pub wounded_length: f64,
}

impl AutostopWoundedLengthEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("AutostopWoundedLengthEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct AutostopStateEvent {
    /// if autostop is enabled
    pub enabled: bool,
    /// if autostop is enabled and alarm is active
    pub enabled_alarm: bool,
    /// limit in mm
    pub limit: f64,
    /// transition state
    pub transition: AutostopTransition,
}

impl AutostopStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("AutostopStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeStateEvent {
    /// mode
    pub mode: Mode,
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
pub struct SpoolStateEvent {
    pub speed_min: f64,
    pub speed_max: f64,
}

impl SpoolStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("SpoolStateEvent", self.clone())
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

pub enum Winder2Events {
    TraversePosition(Event<TraversePositionEvent>),
    TraverseState(Event<TraverseStateEvent>),
    PullerSpeed(Event<PullerSpeedEvent>),
    PullerState(Event<PullerStateEvent>),
    AutostopWoundedlength(Event<AutostopWoundedLengthEvent>),
    AutostopState(Event<AutostopStateEvent>),
    Mode(Event<ModeStateEvent>),
    SpoolRpm(Event<SpoolRpmEvent>),
    SpoolState(Event<SpoolStateEvent>),
    TensionArmAngleEvent(Event<TensionArmAngleEvent>),
    TensionArmStateEvent(Event<TensionArmStateEvent>),
}

#[derive(Debug)]
pub struct Winder2Namespace(Namespace);

impl NamespaceCacheingLogic<Winder2Events> for Winder2Namespace {
    fn emit_cached(&mut self, events: Winder2Events) {
        let event = match events.event_value() {
            Ok(event) => event,
            Err(err) => {
                log::error!(
                    "[{}::emit_cached] Failed to event.event_value(): {:?}",
                    module_path!(),
                    err
                );
                return;
            }
        };
        let buffer_fn = events.event_cache_fn();
        self.0.emit_cached(&event, &buffer_fn);
    }
}

impl Winder2Namespace {
    pub fn new() -> Self {
        Self(Namespace::new())
    }
}

impl CacheableEvents<Winder2Events> for Winder2Events {
    fn event_value(&self) -> Result<GenericEvent, serde_json::Error> {
        match self {
            Winder2Events::TraversePosition(event) => event.try_into(),
            Winder2Events::TraverseState(event) => event.try_into(),
            Winder2Events::PullerSpeed(event) => event.try_into(),
            Winder2Events::PullerState(event) => event.try_into(),
            Winder2Events::AutostopWoundedlength(event) => event.try_into(),
            Winder2Events::AutostopState(event) => event.try_into(),
            Winder2Events::Mode(event) => event.try_into(),
            Winder2Events::SpoolRpm(event) => event.try_into(),
            Winder2Events::SpoolState(event) => event.try_into(),
            Winder2Events::TensionArmAngleEvent(event) => event.try_into(),
            Winder2Events::TensionArmStateEvent(event) => event.try_into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_ten_secs = cache_duration(Duration::from_secs(10), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            Winder2Events::TraversePosition(_) => cache_one_hour,
            Winder2Events::TraverseState(_) => cache_one,
            Winder2Events::PullerSpeed(_) => cache_one_hour,
            Winder2Events::PullerState(_) => cache_one,
            Winder2Events::AutostopWoundedlength(_) => cache_one_hour,
            Winder2Events::AutostopState(_) => cache_one,
            Winder2Events::Mode(_) => cache_one,
            Winder2Events::SpoolRpm(_) => cache_ten_secs,
            Winder2Events::SpoolState(_) => cache_one,
            Winder2Events::TensionArmAngleEvent(_) => cache_one_hour,
            Winder2Events::TensionArmStateEvent(_) => cache_one,
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
            Mutation::AutostopEnable(_) => todo!(),
            Mutation::AutostopEnableAlarm(_) => todo!(),
            Mutation::AutostopSetLimit(_) => todo!(),
            Mutation::AutostopSetTransition(_) => todo!(),
            Mutation::TensionArmAngleZero => self.tension_arm_zero(),
            Mutation::SpoolSetSpeedMax(value) => self.spool_set_speed_max(value),
            Mutation::SpoolSetSpeedMin(value) => self.spool_set_speed_min(value),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut dyn NamespaceInterface {
        &mut self.namespace.0
    }
}
