use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::BottlecapsTestMachine;
use crate::{MachineApi, MachineMessage};

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub outputs: [bool; 8],
    pub inputs: [bool; 8],
    pub stepper_target_speed: i16,
    pub stepper_enabled: bool,
    pub stepper_freq: u8,
    pub stepper_acc_freq: u8,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum BottlecapsTestMachineEvents {
    State(Event<StateEvent>),
    // TODO: add more variants if needed, e.g.:
    //   LiveValues(Event<LiveValuesEvent>),
}

// ----------------------------------------------------------------------------
// Step 3 — Define mutations the frontend can trigger.
//
// Use `#[serde(tag = "action", content = "value")]` so the wire format is:
//   { "action": "SetOutput", "value": { "index": 0, "on": true } }
//
// If your machine is read-only (inputs only), keep the enum empty or with a
// single never-used variant, and return Ok(()) in api_mutate.
// ----------------------------------------------------------------------------
#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    // TODO: add your mutations, e.g.:
    //   SetOutput { index: usize, on: bool },
    //   SetAllOutputs { on: bool },
}

// ----------------------------------------------------------------------------
// Step 4 — The namespace handle (mandatory plumbing, do not change the struct).
// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct BottlecapsTestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<BottlecapsTestMachineEvents> for BottlecapsTestMachineNamespace {
    fn emit(&mut self, events: BottlecapsTestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<BottlecapsTestMachineEvents> for BottlecapsTestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            BottlecapsTestMachineEvents::State(event) => event.clone().into(),
            // TODO: add arms for every new variant you add above
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        // `cache_first_and_last_event` keeps the very first and the most recent
        // event in the cache so new subscribers always get a value immediately.
        cache_first_and_last_event()
    }
}

// ----------------------------------------------------------------------------
// Step 5 — Implement MachineApi (mandatory plumbing + your mutation logic).
// ----------------------------------------------------------------------------
impl MachineApi for BottlecapsTestMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        // TODO: deserialize and dispatch mutations:
        //
        // let mutation: Mutation = serde_json::from_value(request_body)?;
        // match mutation {
        //     Mutation::SetOutput { index, on } => self.set_output(index, on),
        //     Mutation::SetAllOutputs { on } => { /* ... */ }
        // }

        // If this machine is read-only, just return Ok(()) without parsing:
        let _ = request_body;
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
