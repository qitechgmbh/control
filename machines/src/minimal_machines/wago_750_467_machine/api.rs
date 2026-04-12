use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::Serialize;
use serde_json::Value;

use super::Wago750_467Machine;
use crate::{MachineApi, MachineMessage};

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub voltages: [f64; 2],
    pub normalized: [f32; 2],
    pub raw_words: [u16; 2],
    pub wiring_errors: [bool; 2],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum Wago750_467MachineEvents {
    State(Event<StateEvent>),
}

#[derive(Debug, Clone)]
pub struct Wago750_467MachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<Wago750_467MachineEvents> for Wago750_467MachineNamespace {
    fn emit(&mut self, events: Wago750_467MachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<Wago750_467MachineEvents> for Wago750_467MachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Wago750_467MachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for Wago750_467Machine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, _request_body: Value) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
