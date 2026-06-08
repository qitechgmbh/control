use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Wago750_531Machine;
use crate::{MachineApi, MachineMessage, MachineValues};

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub outputs_on: [bool; 4],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum Wago750_531MachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetOutput { index: usize, on: bool },
    SetAllOutputs { on: bool },
}

#[derive(Debug, Clone)]
pub struct Wago750_531MachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<Wago750_531MachineEvents> for Wago750_531MachineNamespace {
    fn emit(&mut self, events: Wago750_531MachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<Wago750_531MachineEvents> for Wago750_531MachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Wago750_531MachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for Wago750_531Machine {
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
                        live_values: serde_json::Value::Null,
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
            Mutation::SetOutput { index, on } => self.set_output(index, on),
            Mutation::SetAllOutputs { on } => self.set_all_outputs(on),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
