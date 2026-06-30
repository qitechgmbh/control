use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};

use crate::{MachineApi, minimal_machines::wago_750_430_di_machine::Wago750_430DiMachine};

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub inputs: [bool; 8],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum Wago750_430DiMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {}

#[derive(Debug, Clone)]
pub struct Wago750_430DiMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<Wago750_430DiMachineEvents> for Wago750_430DiMachineNamespace {
    fn emit(&mut self, events: Wago750_430DiMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<Wago750_430DiMachineEvents> for Wago750_430DiMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Wago750_430DiMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for Wago750_430DiMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<crate::MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, _value: serde_json::Value) -> Result<(), anyhow::Error> {
        // WAGO 750-430 is a digital input only module - no mutations supported
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
