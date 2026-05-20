use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::TestMachine;
use crate::{MachineApi, MachineMessage, MachineValues};

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub led_on: [bool; 4],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum TestMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetLed { index: usize, on: bool },
    SetAllLeds { on: bool },
}

#[derive(Debug, Clone)]
pub struct TestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<TestMachineEvents> for TestMachineNamespace {
    fn emit(&mut self, events: TestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<TestMachineEvents> for TestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            TestMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for TestMachine {
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
            Mutation::SetLed { index, on } => self.set_led(index, on),
            Mutation::SetAllLeds { on } => self.set_all_leds(on),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
