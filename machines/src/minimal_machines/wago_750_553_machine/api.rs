use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Wago750_553Machine;
use crate::{MachineApi, MachineMessage};

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub outputs: [f32; 4],
    pub outputs_ma: [f32; 4],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum Wago750_553MachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetOutput { index: usize, value: f32 },
    SetAllOutputs { value: f32 },
}

#[derive(Debug, Clone)]
pub struct Wago750_553MachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<Wago750_553MachineEvents> for Wago750_553MachineNamespace {
    fn emit(&mut self, events: Wago750_553MachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<Wago750_553MachineEvents> for Wago750_553MachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Wago750_553MachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for Wago750_553Machine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetOutput { index, value } => self.set_output(index, value),
            Mutation::SetAllOutputs { value } => self.set_all_outputs(value),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
