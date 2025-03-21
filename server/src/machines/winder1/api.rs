use std::time::Duration;

use super::WinderV1;
use crate::{
    ethercat::device_identification::MachineIdentificationUnique,
    machines::MachineApi,
    socketio::{
        event::{Event, EventBuilder, GenericEvent},
        room::{
            room::{
                cache_duration, cache_one_event, CacheFn, CacheableEvents, Room, RoomCacheingLogic,
                RoomInterface,
            },
            room_id::RoomId,
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
    Pull,
    Wind,
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

    // Mode
    ModeSet(Mode),
}

#[derive(Serialize, Debug, Clone)]
pub struct TraversePositionEvent {
    /// position in mm
    pub position: f64,
}

impl EventBuilder<TraversePositionEvent> for TraversePositionEvent {
    fn name(&self) -> String {
        "TraversePositionEvent".to_string()
    }
    fn build(&self) -> Event<Self> {
        Event::data(&self.name(), self.clone())
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

impl EventBuilder<TraverseStateEvent> for TraverseStateEvent {
    fn name(&self) -> String {
        "TraverseStateEvent".to_string()
    }
    fn build(&self) -> Event<Self> {
        Event::data(&self.name(), self.clone())
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

impl EventBuilder<PullerStateEvent> for PullerStateEvent {
    fn name(&self) -> String {
        "PullerStateEvent".to_string()
    }
    fn build(&self) -> Event<Self> {
        Event::data(&self.name(), self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PullerSpeedEvent {
    /// speed in mm/s
    pub speed: f64,
}

impl EventBuilder<PullerSpeedEvent> for PullerSpeedEvent {
    fn name(&self) -> String {
        "PullerSpeedEvent".to_string()
    }
    fn build(&self) -> Event<Self> {
        Event::data(&self.name(), self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct AutostopWoundedLengthEvent {
    /// wounded length in mm
    pub wounded_length: f64,
}

impl EventBuilder<AutostopWoundedLengthEvent> for AutostopWoundedLengthEvent {
    fn name(&self) -> String {
        "AutostopWoundedLengthEvent".to_string()
    }
    fn build(&self) -> Event<Self> {
        Event::data(&self.name(), self.clone())
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

impl EventBuilder<AutostopStateEvent> for AutostopStateEvent {
    fn name(&self) -> String {
        "AutostopStateEvent".to_string()
    }
    fn build(&self) -> Event<Self> {
        Event::data(&self.name(), self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeStateEvent {
    /// mode
    pub mode: Mode,
}

impl EventBuilder<ModeStateEvent> for ModeStateEvent {
    fn name(&self) -> String {
        "ModeStateEvent".to_string()
    }
    fn build(&self) -> Event<Self> {
        Event::data(&self.name(), self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct MeasurementsWindingRpmEvent {
    /// rpm
    pub rpm: f64,
}

impl EventBuilder<MeasurementsWindingRpmEvent> for MeasurementsWindingRpmEvent {
    fn name(&self) -> String {
        "MeasurementsWindingRpmEvent".to_string()
    }
    fn build(&self) -> Event<Self> {
        Event::data(&self.name(), self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct MeasurementsTensionArmEvent {
    /// degree
    pub degree: f32,
}

impl EventBuilder<MeasurementsTensionArmEvent> for MeasurementsTensionArmEvent {
    fn name(&self) -> String {
        "MeasurementsTensionArmEvent".to_string()
    }
    fn build(&self) -> Event<Self> {
        Event::data(&self.name(), self.clone())
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
    MeasurementsWindingRpm(Event<MeasurementsWindingRpmEvent>),
    MeasurementsTensionArm(Event<MeasurementsTensionArmEvent>),
}

#[derive(Debug)]
pub struct Winder1Room(Room);

impl RoomCacheingLogic<Winder1Events> for Winder1Room {
    fn emit_cached(&mut self, events: Winder1Events) {
        let event = events.event_value();
        let buffer_fn = events.event_cache_fn();
        self.0.emit_cached(&event, buffer_fn);
    }
}

impl Winder1Room {
    pub fn new(machine_identification_unique: MachineIdentificationUnique) -> Self {
        Self(Room::new(RoomId::Machine(machine_identification_unique)))
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
            Winder1Events::MeasurementsWindingRpm(event) => event.into(),
            Winder1Events::MeasurementsTensionArm(event) => event.into(),
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
            Winder1Events::MeasurementsWindingRpm(_) => cache_ten_secs,
            Winder1Events::MeasurementsTensionArm(_) => cache_one_hour,
        }
    }
}

impl MachineApi for WinderV1 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::TraverseEnableLaserpointer(enable) => self.set_laser(enable),
            _ => anyhow::bail!(
                "[{}::MachineApi/WinderV1::api_mutate] Mutation {} not implemented",
                module_path!(),
                serde_json::to_string(&mutation)?
            ),
        }
        Ok(())
    }

    fn api_event_room(&mut self) -> &mut dyn RoomInterface {
        &mut self.room.0
    }
}
