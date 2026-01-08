use super::MinimalBottleSorter;
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
    pub stepper_enabled: bool,
    pub stepper_speed: f64,
    pub stepper_direction: bool,
    pub outputs: [bool; 8],
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct LiveValuesEvent {
    pub stepper_actual_speed: f64,
    pub stepper_position: i128,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

pub enum MinimalBottleSorterEvents {
    State(Event<StateEvent>),
    LiveValues(Event<LiveValuesEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetStepperSpeed { speed: f64 },
    SetStepperDirection { forward: bool },
    SetStepperEnabled { enabled: bool },
    PulseOutput { index: usize, duration_ms: u32 },
}

#[derive(Debug, Clone)]
pub struct MinimalBottleSorterNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<MinimalBottleSorterEvents> for MinimalBottleSorterNamespace {
    fn emit(&mut self, events: MinimalBottleSorterEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<MinimalBottleSorterEvents> for MinimalBottleSorterEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            MinimalBottleSorterEvents::State(event) => event.clone().into(),
            MinimalBottleSorterEvents::LiveValues(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for MinimalBottleSorter {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetStepperSpeed { speed } => self.set_stepper_speed(speed),
            Mutation::SetStepperDirection { forward } => self.set_stepper_direction(forward),
            Mutation::SetStepperEnabled { enabled } => self.set_stepper_enabled(enabled),
            Mutation::PulseOutput { index, duration_ms } => self.pulse_output(index, duration_ms),
        }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
