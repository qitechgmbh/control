use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};

use crate::{MachineApi, minimal_machines::digital_input_test_machine::DigitalInputTestMachine};

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
    fn api_get_sender(&self) -> smol::channel::Sender<crate::MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, _value: serde_json::Value) -> Result<(), anyhow::Error> {
        //Digital Input Test Machine does not Set Values
        // let mutation: Mutation = serde_json::from_value(value)?;
        // match mutation {
        //     Mutation::SetLed { index, on } => self.set_led(index, on),
        // }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
