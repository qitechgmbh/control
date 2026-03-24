use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};

use crate::{MachineApi, minimal_machines::wago_8ch_dio_test_machine::Wago8chDigitalIOTestMachine};

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub digital_input: [bool; 8],
    pub digital_output: [bool; 8],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}
pub enum Wago8chDigitalIOTestMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetOutput { index: usize, value: bool },
}
#[derive(Debug, Clone)]
pub struct Wago8chDigitalIOTestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<Wago8chDigitalIOTestMachineEvents>
    for Wago8chDigitalIOTestMachineNamespace
{
    fn emit(&mut self, events: Wago8chDigitalIOTestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<Wago8chDigitalIOTestMachineEvents> for Wago8chDigitalIOTestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Wago8chDigitalIOTestMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for Wago8chDigitalIOTestMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<crate::MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, value: serde_json::Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(value)?;
        match mutation {
            Mutation::SetOutput { index, value } => self.set_output(index, value),
        }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
