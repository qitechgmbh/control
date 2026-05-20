use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use control_core_derive::BuildEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;

use super::MockMachine;
use crate::{MachineApi, MachineMessage, MachineValues};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Standby,
    Running,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub amplitude_sum: f64,
    pub amplitude1: f64,
    pub amplitude2: f64,
    pub amplitude3: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, BuildEvent)]
pub struct StateEvent {
    pub is_default_state: bool,
    pub frequency1: f64,
    pub frequency2: f64,
    pub frequency3: f64,
    pub mode_state: ModeState,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ModeState {
    pub mode: Mode,
}

pub enum MockEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct MockMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl CacheableEvents<MockEvents> for MockEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl NamespaceCacheingLogic<MockEvents> for MockMachineNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: MockEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    SetFrequency1(f64),
    SetFrequency2(f64),
    SetFrequency3(f64),
    SetMode(Mode),
}

impl MachineApi for MockMachine {
    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => {
                self.namespace.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => {
                let _res = self.api_mutate(value);
            }
            MachineMessage::RequestValues(sender) => {
                sender
                    .send(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
            }
        }
    }

    fn get_api_sender(&self) -> tokio::sync::mpsc::Sender<MachineMessage> {
        self.sender.clone()
    }

    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(value)?;
        match mutation {
            Mutation::SetFrequency1(f) => self.set_frequency1(f),
            Mutation::SetFrequency2(f) => self.set_frequency2(f),
            Mutation::SetFrequency3(f) => self.set_frequency3(f),
            Mutation::SetMode(mode) => self.set_mode(mode),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
