use std::time::Duration;

use super::{Winder2, Winder2Mode};
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            cache_duration, cache_one_event, CacheFn, CacheableEvents, Namespace,
            NamespaceCacheingLogic, NamespaceInterface,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PullerRegulation {
    Speed,
    Diameter,
}

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
    TraverseSetLimitOuter(f64),
    TraverseSetLimitInner(f64),
    TraverseGotoLimitOuter,
    TraverseGotoLimitInner,
    TraverseEnableLaserpointer(bool),
    TraverseGotoHome(f64),

    // Puller
    /// on = speed, off = stop
    PullerSetRegulation(PullerRegulation),
    PullerSetTargetSpeed(f64),
    PullerSetTargetDiameter(f64),

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
    pub position: f64,
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
    /// is at position in
    pub is_in: bool,
    /// is at position out
    pub is_out: bool,
    /// is going to position in
    pub is_going_in: bool,
    /// is going to position out
    pub is_going_out: bool,
    /// if is homed
    pub is_homed: bool,
    /// if is homing
    pub is_going_home: bool,
    /// laserpointer is on
    pub laserpointer: bool,
}

impl TraverseStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("TraverseStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PullerStateEvent {
    /// speed in mm/s
    pub speed: f64,
    /// regulation type
    pub regulation: PullerRegulation,
    /// target speed in mm/s
    pub target_speed: f64,
    /// target diameter in mm
    pub target_diameter: f64,
}

impl PullerStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("PullerStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PullerSpeedEvent {
    /// speed in mm/s
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
    pub rpm: f32,
}

impl SpoolRpmEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("SpoolRpmEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SpoolStateEvent {
    pub speed_min: f32,
    pub speed_max: f32,
}

impl SpoolStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("SpoolStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct TensionArmAngleEvent {
    /// degree
    pub degree: f32,
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

pub enum Winder1Events {
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
pub struct Winder1Namespace(Namespace);

impl NamespaceCacheingLogic<Winder1Events> for Winder1Namespace {
    fn emit_cached(&mut self, events: Winder1Events) {
        let event = events.event_value();
        let buffer_fn = events.event_cache_fn();
        self.0.emit_cached(&event, buffer_fn);
    }
}

impl Winder1Namespace {
    pub fn new() -> Self {
        Self(Namespace::new())
    }
}

impl CacheableEvents<Winder1Events> for Winder1Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            Winder1Events::TraversePosition(event) => event.into(),
            Winder1Events::TraverseState(event) => event.into(),
            Winder1Events::PullerSpeed(event) => event.into(),
            Winder1Events::PullerState(event) => event.into(),
            Winder1Events::AutostopWoundedlength(event) => event.into(),
            Winder1Events::AutostopState(event) => event.into(),
            Winder1Events::Mode(event) => event.into(),
            Winder1Events::SpoolRpm(event) => event.into(),
            Winder1Events::SpoolState(event) => event.into(),
            Winder1Events::TensionArmAngleEvent(event) => event.into(),
            Winder1Events::TensionArmStateEvent(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60));
        let cache_ten_secs = cache_duration(Duration::from_secs(10));
        let cache_one = cache_one_event();

        match self {
            Winder1Events::TraversePosition(_) => cache_one_hour,
            Winder1Events::TraverseState(_) => cache_one,
            Winder1Events::PullerSpeed(_) => cache_one_hour,
            Winder1Events::PullerState(_) => cache_one,
            Winder1Events::AutostopWoundedlength(_) => cache_one_hour,
            Winder1Events::AutostopState(_) => cache_one,
            Winder1Events::Mode(_) => cache_one,
            Winder1Events::SpoolRpm(_) => cache_ten_secs,
            Winder1Events::SpoolState(_) => cache_one,
            Winder1Events::TensionArmAngleEvent(_) => cache_one_hour,
            Winder1Events::TensionArmStateEvent(_) => cache_one,
        }
    }
}

impl MachineApi for Winder2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::TraverseEnableLaserpointer(enable) => self.set_laser(enable),
            Mutation::ModeSet(mode) => self.set_mode(&mode.into()),
            Mutation::TraverseSetLimitOuter(_) => todo!(),
            Mutation::TraverseSetLimitInner(_) => todo!(),
            Mutation::TraverseGotoLimitOuter => todo!(),
            Mutation::TraverseGotoLimitInner => todo!(),
            Mutation::TraverseGotoHome(_) => todo!(),
            Mutation::PullerSetRegulation(_) => todo!(),
            Mutation::PullerSetTargetSpeed(_) => todo!(),
            Mutation::PullerSetTargetDiameter(_) => todo!(),
            Mutation::AutostopEnable(_) => todo!(),
            Mutation::AutostopEnableAlarm(_) => todo!(),
            Mutation::AutostopSetLimit(_) => todo!(),
            Mutation::AutostopSetTransition(_) => todo!(),
            Mutation::TensionArmAngleZero => self.tension_arm_zero(),
            Mutation::SpoolSetSpeedMax(value) => self.spool_set_speed_max(value as f32),
            Mutation::SpoolSetSpeedMin(value) => self.spool_set_speed_min(value as f32),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut dyn NamespaceInterface {
        &mut self.namespace.0
    }
}
