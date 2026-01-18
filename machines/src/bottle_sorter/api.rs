use super::BottleSorter;
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
    pub outputs: [bool; 8],
    pub stepper_speed_mm_s: f64,
    pub stepper_enabled: bool,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct LiveValuesEvent {
    pub stepper_position: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

pub enum BottleSorterEvents {
    State(Event<StateEvent>),
    LiveValues(Event<LiveValuesEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetStepperSpeed { speed_mm_s: f64 },
    SetStepperEnabled { enabled: bool },
    PulseOutput { index: usize },
}

#[derive(Debug, Clone)]
pub struct BottleSorterNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<BottleSorterEvents> for BottleSorterNamespace {
    fn emit(&mut self, events: BottleSorterEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<BottleSorterEvents> for BottleSorterEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            BottleSorterEvents::State(event) => event.clone().into(),
            BottleSorterEvents::LiveValues(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for BottleSorter {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetStepperSpeed { speed_mm_s } => self.set_stepper_speed(speed_mm_s),
            Mutation::SetStepperEnabled { enabled } => self.set_stepper_enabled(enabled),
            Mutation::PulseOutput { index } => self.pulse_output(index),
        }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
