use std::{sync::Arc, time::Duration};

use super::{BufferV1, BufferV1Mode};
use control_core::{
    machines::{api::MachineApi, identification::MachineIdentificationUnique},
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

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    /// mode state
    pub mode_state: ModeState,
    /// connected machine state
    pub connected_machine_state: ConnectedMachineState,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Mode {
    Standby,
    FillingBuffer,
    EmptyingBuffer,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConnectedMachineState {
    /// Connected Machine
    pub machine_identification_unique: Option<MachineIdentificationUnique>,
    pub is_available: bool,
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    // Mode
    SetBufferMode(BufferV1Mode),

    // Connected Machine
    SetConnectedMachine(MachineIdentificationUnique),

    // Disconnect Machine
    DisconnectMachine(MachineIdentificationUnique),
}

#[derive(Debug)]
pub struct Buffer1Namespace {
    pub namespace: Namespace,
}

impl NamespaceCacheingLogic<BufferV1Events> for Buffer1Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: BufferV1Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        self.namespace.emit(event, &buffer_fn);
    }
}

impl Buffer1Namespace {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
        }
    }
}

impl CacheableEvents<BufferV1Events> for BufferV1Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            BufferV1Events::LiveValues(event) => event.into(),
            BufferV1Events::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            BufferV1Events::LiveValues(_) => cache_one_hour,
            BufferV1Events::State(_) => cache_one,
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
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.namespace
    }
}
