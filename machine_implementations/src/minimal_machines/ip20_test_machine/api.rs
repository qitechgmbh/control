use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::IP20TestMachine;
use crate::{MachineApi, MachineMessage, MachineValues};

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub outputs: [bool; 8],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct LiveValuesEvent {
    pub inputs: [bool; 8],
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

pub enum IP20TestMachineEvents {
    State(Event<StateEvent>),
    LiveValues(Event<LiveValuesEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetOutput { index: usize, on: bool },
    SetAllOutputs { on: bool },
}

#[derive(Debug, Clone)]
pub struct IP20TestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<IP20TestMachineEvents> for IP20TestMachineNamespace {
    fn emit(&mut self, events: IP20TestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<IP20TestMachineEvents> for IP20TestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            IP20TestMachineEvents::State(event) => event.clone().into(),
            IP20TestMachineEvents::LiveValues(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for IP20TestMachine {
    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
                self.emit_live_values();
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
            Mutation::SetOutput { index, on } => self.set_output(index, on),
            Mutation::SetAllOutputs { on } => self.set_all_outputs(on),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
