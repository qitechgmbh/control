use super::{BufferV1, BufferV1Mode};
use crate::{MachineApi, MachineMessage, machine_identification::MachineIdentificationUnique};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_one_event},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use std::sync::Arc;
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
    pub mode_state: ModeState, // connected machine state
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
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<BufferV1Events> for Buffer1Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: BufferV1Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
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
        let cache_one = cache_one_event();

        match self {
            Self::LiveValues(_) => cache_one,
            Self::State(_) => cache_one,
        }
    }
}

impl MachineApi for BufferV1 {
    fn api_get_sender(&self) -> Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetBufferMode(mode) => self.set_mode_state(mode),
            Mutation::SetConnectedMachine(_machine_identification_unique) => {}
            Mutation::DisconnectMachine(_machine_identification_unique) => {}
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
