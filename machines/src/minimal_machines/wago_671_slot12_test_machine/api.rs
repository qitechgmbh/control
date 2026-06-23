use super::Wago671Slot12TestMachine;
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
pub struct AxisStateEvent {
    pub enabled: bool,
    pub target_speed: i16,
    pub target_velocity: i16,
    pub target_speed_steps_per_second: i32,
    pub actual_velocity: i16,
    pub actual_speed_steps_per_second: f64,
    pub acceleration: u16,
    pub freq: u8,
    pub acc_freq: u8,
    pub raw_position: i64,
    pub control_byte1: u8,
    pub control_byte2: u8,
    pub control_byte3: u8,
    pub status_byte1: u8,
    pub status_byte2: u8,
    pub status_byte3: u8,
    pub speed_mode_ack: bool,
    pub start_ack: bool,
    pub di1: bool,
    pub di2: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub slot1: AxisStateEvent,
    pub slot2: AxisStateEvent,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum Wago671Slot12TestMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetTargetSpeed { axis: u8, target: i16 },
    SetEnabled { axis: u8, enabled: bool },
    SetFreq { axis: u8, factor: u8 },
    SetAccFreq { axis: u8, factor: u8 },
    SetAcceleration { axis: u8, acceleration: u16 },
}

#[derive(Debug, Clone)]
pub struct Wago671Slot12TestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<Wago671Slot12TestMachineEvents> for Wago671Slot12TestMachineNamespace {
    fn emit(&mut self, events: Wago671Slot12TestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<Wago671Slot12TestMachineEvents> for Wago671Slot12TestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Wago671Slot12TestMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for Wago671Slot12TestMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetTargetSpeed { axis, target } => self.set_target_speed(axis, target),
            Mutation::SetEnabled { axis, enabled } => self.set_enabled(axis, enabled),
            Mutation::SetFreq { axis, factor } => self.set_freq(axis, factor),
            Mutation::SetAccFreq { axis, factor } => self.set_acc_freq(axis, factor),
            Mutation::SetAcceleration { axis, acceleration } => {
                self.set_acceleration(axis, acceleration)
            }
        }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
