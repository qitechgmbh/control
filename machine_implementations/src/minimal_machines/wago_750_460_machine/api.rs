use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::Serialize;
use serde_json::Value;

use super::Wago750_460Machine;
use crate::{MachineApi, MachineMessage};

// ----------------------------------------------------------------------------
// StateEvent — broadcast to frontend on every cycle
//
// `temperatures[i]` is `Some(°C)` when the channel reads normally, or `None`
// when a wire-break / range error is active on that channel.
// `errors[i]` is set when the measurement value saturates at the range limits
// (overrange ≥ +850.0 °C or underrange ≤ -200.0 °C), indicating a sensor fault.
// ----------------------------------------------------------------------------
#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    /// Temperature in °C for each channel; None when sensor error is active.
    pub temperatures: [Option<f64>; 4],
    /// Wire-break / overrange error flag per channel.
    pub errors: [bool; 4],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum Wago750_460MachineEvents {
    State(Event<StateEvent>),
}

#[derive(Debug, Clone)]
pub struct Wago750_460MachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<Wago750_460MachineEvents> for Wago750_460MachineNamespace {
    fn emit(&mut self, events: Wago750_460MachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<Wago750_460MachineEvents> for Wago750_460MachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Wago750_460MachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

// This machine is read-only — no mutations.
impl MachineApi for Wago750_460Machine {
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
