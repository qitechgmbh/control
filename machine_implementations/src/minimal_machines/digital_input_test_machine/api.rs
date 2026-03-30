use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use std::sync::Arc;

use crate::MachineApi;
use crate::{
    MachineMessage, minimal_machines::digital_input_test_machine::DigitalInputTestMachine,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub led_on: [bool; 4],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}
pub enum DigitalInputTestMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetLed { index: usize, on: bool },
}
#[derive(Debug, Clone)]
pub struct DigitalInputTestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<DigitalInputTestMachineEvents> for DigitalInputTestMachineNamespace {
    fn emit(&mut self, events: DigitalInputTestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<DigitalInputTestMachineEvents> for DigitalInputTestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            DigitalInputTestMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for DigitalInputTestMachine {
    fn get_api_sender(&self) -> Sender<MachineMessage> {
        self.sender.clone()
    }

    fn api_mutate(&mut self, _value: serde_json::Value) -> Result<(), anyhow::Error> {
        //let mutation: Mutation = serde_json::from_value(value)?;
        // match mutation {
        //     Mutation::SetLed { index, on } => self.set_led(index, on),
        // }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }

    fn act_machine_message(&mut self, msg: crate::MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace = DigitalInputTestMachineNamespace {
                    namespace: Some(namespace),
                }
            }
            MachineMessage::UnsubscribeNamespace => {
                self.namespace.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => (),
            MachineMessage::RequestValues(sender) => (),
        }
    }
}
