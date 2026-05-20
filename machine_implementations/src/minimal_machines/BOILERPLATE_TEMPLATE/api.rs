// ============================================================================
// api.rs — Events, Mutations, Namespace, MachineApi impl
// ============================================================================
// Contains:
//   • StateEvent       — payload broadcast to socket.io subscribers
//   • MyMachineEvents  — enum wrapping every event variant this machine emits
//   • Mutation         — actions the frontend can POST to this machine
//   • MyMachineNamespace — socket.io namespace handle (mandatory plumbing)
//   • MachineApi impl  — message dispatch + mutation handling
// ============================================================================

use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::MyMachine;
use crate::{MachineApi, MachineMessage, MachineValues};

// ----------------------------------------------------------------------------
// Step 1 — Define the state broadcast to the frontend.
// Every field the UI displays goes here. Must be Serialize + Clone.
// ----------------------------------------------------------------------------
#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    // TODO: state fields, e.g.:
    // pub outputs: [bool; 4],
    // pub inputs:  [bool; 4],
    // pub value:   f64,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

// ----------------------------------------------------------------------------
// Step 2 — Event variants this machine can emit.
// One `State` variant is enough for most minimal machines. Add more (e.g.
// `LiveValues`, `Alert`) only if the frontend needs to distinguish them.
// ----------------------------------------------------------------------------
pub enum MyMachineEvents {
    State(Event<StateEvent>),
    // TODO: add more variants if needed.
}

// ----------------------------------------------------------------------------
// Step 3 — Frontend → backend mutations.
//
// Wire format with `#[serde(tag = "action", content = "value")]`:
//   { "action": "SetOutput", "value": { "index": 0, "value": true } }
//
// If the machine is read-only, leave the enum empty AND remove the
// `serde_json::from_value` call in `api_mutate` below (returning Ok(()) directly).
// ----------------------------------------------------------------------------
#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    // TODO: mutations, e.g.:
    // SetOutput { index: usize, value: bool },
}

// ----------------------------------------------------------------------------
// Step 4 — Namespace handle (mandatory plumbing).
// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct MyMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<MyMachineEvents> for MyMachineNamespace {
    fn emit(&mut self, events: MyMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<MyMachineEvents> for MyMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            MyMachineEvents::State(event) => event.clone().into(),
            // TODO: add an arm for every new variant.
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        // Caches the first AND most recent event so new subscribers get a
        // value immediately on connect.
        cache_first_and_last_event()
    }
}

// ----------------------------------------------------------------------------
// Step 5 — MachineApi: message dispatch + mutation handling.
// The four arms below are required boilerplate. Only `HttpApiJsonRequest` /
// `api_mutate` need machine-specific code.
// ----------------------------------------------------------------------------
impl MachineApi for MyMachine {
    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                // Push current state so the new subscriber doesn't wait for
                // the next ~33 ms emit cycle.
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => {
                self.namespace.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => {
                let _res = self.api_mutate(value);
            }
            MachineMessage::RequestValues(sender) => {
                sender
                    .send(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::Value::Null,
                    })
                    .expect("Failed to send values");
            }
        }
    }

    fn get_api_sender(&self) -> tokio::sync::mpsc::Sender<MachineMessage> {
        self.sender.clone()
    }

    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error> {
        // For read-only machines, delete the lines below and just `Ok(())`.
        let mutation: Mutation = serde_json::from_value(value)?;
        match mutation {
            // TODO: dispatch arms, e.g.:
            // Mutation::SetOutput { index, value } => self.set_output(index, value),
            #[allow(unreachable_patterns)]
            _ => {}
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
