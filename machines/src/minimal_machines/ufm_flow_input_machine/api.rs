use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{MachineApi, MachineMessage};
use super::UfmFlowInputMachine;

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    /// Flow rate in liters per hour.
    pub flow_lph: f64,
    /// Accumulated total volume in cubic meters.
    pub total_volume_m3: f64,
    /// True when the sensor reports an error (no water / low amplitude).
    pub error: bool,
    /// Raw pulse count since the machine started.
    pub total_pulses: u64,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum UfmFlowInputMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {}

#[derive(Debug, Clone)]
pub struct UfmFlowInputMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<UfmFlowInputMachineEvents> for UfmFlowInputMachineNamespace {
    fn emit(&mut self, events: UfmFlowInputMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<UfmFlowInputMachineEvents> for UfmFlowInputMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            UfmFlowInputMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for UfmFlowInputMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let _ = request_body;
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
