use std::{sync::Arc, time::Duration};

use crate::machines::buffer1::puller_speed_controller::PullerRegulationMode;

use super::{BufferV1, BufferV1Mode};
use control_core::{
    machines::{
        api::MachineApi, connection::MachineCrossConnectionState,
        identification::MachineIdentificationUnique,
    },
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
use smol::lock::Mutex;
use tracing::instrument;

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    /// puller speed in m/min
    pub puller_speed: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    /// mode state
    pub mode_state: ModeState,
    /// puller state
    pub puller_state: PullerState,
    /// connected machine state
    pub connected_machine_state: MachineCrossConnectionState,
    /// CurrentInputSpeed
    pub current_input_speed_state: CurrentInputSpeedState,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum BufferV1Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeState {
    pub mode: BufferV1Mode,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Standby,
    FillingBuffer,
    EmptyingBuffer,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ConnectedMachineState {
    /// Connected Machine
    pub machine_identification_unique: Option<MachineIdentificationUnique>,
    pub is_available: bool,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CurrentInputSpeedState {
    pub current_input_speed: f64,
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    // Mode
    SetBufferMode(BufferV1Mode),

    // Connected Machine
    SetConnectedMachine(MachineIdentificationUnique),

    // Disconnect Machine
    DisconnectMachine(MachineIdentificationUnique),

    // Set current input speed
    SetCurrentInputSpeed(f64),

    // Puller
    /// on = speed, off = stop
    SetPullerRegulationMode(PullerRegulationMode),
    SetPullerTargetSpeed(f64),
    SetPullerTargetDiameter(f64),
    SetPullerForward(bool),
}

#[derive(Debug)]
pub struct Buffer1Namespace {
    pub namespace: Arc<Mutex<Namespace>>,
}

impl NamespaceCacheingLogic<BufferV1Events> for Buffer1Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: BufferV1Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        let mut namespace = self.namespace.lock_blocking();
        namespace.emit(event, &buffer_fn);
    }
}

impl CacheableEvents<Self> for BufferV1Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            Self::LiveValues(_) => cache_one_hour,
            Self::State(_) => cache_one,
        }
    }
}

impl MachineApi for BufferV1 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetBufferMode(mode) => self.set_mode_state(mode),
            Mutation::SetConnectedMachine(machine_identification_unique) => {
                self.set_connected_winder(machine_identification_unique);
            }
            Mutation::DisconnectMachine(machine_identification_unique) => {
                self.disconnect_winder(machine_identification_unique);
            }
            Mutation::SetCurrentInputSpeed(speed) => self.set_current_input_speed(speed),
            Mutation::SetPullerRegulationMode(regulation) => self.puller_set_regulation(regulation),
            Mutation::SetPullerTargetSpeed(value) => self.puller_set_target_speed(value),
            Mutation::SetPullerTargetDiameter(_) => todo!(),
            Mutation::SetPullerForward(value) => self.puller_set_forward(value),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>> {
        self.namespace.namespace.clone()
    }
}
