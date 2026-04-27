use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};

use crate::{MachineApi, minimal_machines::ufm_flow_test_machine::UfmFlowTestMachine};

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub flow_lph: f64,
    pub total_volume_m3: f64,
    pub sensor_error: bool,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum UfmFlowTestMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {}

#[derive(Debug, Clone)]
pub struct UfmFlowTestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<UfmFlowTestMachineEvents> for UfmFlowTestMachineNamespace {
    fn emit(&mut self, events: UfmFlowTestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<UfmFlowTestMachineEvents> for UfmFlowTestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            UfmFlowTestMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for UfmFlowTestMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<crate::MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, _value: serde_json::Value) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
