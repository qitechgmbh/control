use super::WagoSerialMachine;
use crate::{MachineApi, MachineMessage};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub current_message : Option<String>,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum WagoSerialMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
pub enum Mutation {    
    SendMessage(String),
}

#[derive(Debug, Clone)]
pub struct WagoSerialMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<WagoSerialMachineEvents> for WagoSerialMachineNamespace {
    fn emit(&mut self, events: WagoSerialMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<WagoSerialMachineEvents> for WagoSerialMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            WagoSerialMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for WagoSerialMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SendMessage(msg) => self.send_message(msg),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
