use super::WagoDOTestMachine;
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
    pub led_on: [bool; 8],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum WagoDOTestMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetLed { index: usize, on: bool },
    SetAllLeds { on: bool },
}

#[derive(Debug, Clone)]
pub struct WagoDOTestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<WagoDOTestMachineEvents> for WagoDOTestMachineNamespace {
    fn emit(&mut self, events: WagoDOTestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<WagoDOTestMachineEvents> for WagoDOTestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            WagoDOTestMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for WagoDOTestMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetLed { index, on } => self.set_led(index, on),
            Mutation::SetAllLeds { on } => self.set_all_leds(on),
        }

        for (led, &on) in self.douts.iter().zip(self.led_on.iter()) {
            led.set(on);
        }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
