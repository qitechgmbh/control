use super::WagoDioSeparate;
use crate::{MachineApi, MachineMessage};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic,
        cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub inputs: [bool; 8],
    pub led_on: [bool; 8],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum WagoDioSeparateEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetLed { index: usize, on: bool },
    SetAllLeds { on: bool },
}

#[derive(Debug, Clone)]
pub struct WagoDioSeparateNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<WagoDioSeparateEvents> for WagoDioSeparateNamespace {
    fn emit(&mut self, events: WagoDioSeparateEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<WagoDioSeparateEvents> for WagoDioSeparateEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            WagoDioSeparateEvents::State(event) => event.clone().into(),
        }
    }
    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for WagoDioSeparate {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
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