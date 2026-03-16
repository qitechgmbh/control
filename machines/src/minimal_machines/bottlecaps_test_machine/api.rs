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
    pub inputs: [bool; 8],
    pub outputs: [bool; 8],
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

#[derive(Serialize, Debug, Clone)]
pub struct LiveValuesEvent {
    pub inputs: [bool; 8],
}

pub enum BottlecapsTestMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetOutput { index: usize, on: bool },
    SetStepperTargetSpeed { target: i16 },
    SetStepperEnabled { enabled: bool },
    SetStepperFreq { factor: u8 },
    SetStepperAccFreq { factor: u8 },
}

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
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for BottlecapsTestMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetStepperTargetSpeed { target } => self.stepper_set_target_speed(target),
            Mutation::SetStepperEnabled { enabled } => self.stepper_set_enabled(enabled),
            Mutation::SetStepperFreq { factor } => self.stepper_set_freq(factor),
            Mutation::SetStepperAccFreq { factor } => self.stepper_set_acc_freq(factor),
            Mutation::SetOutput { index, on } => self.set_output(index, on),
        }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
