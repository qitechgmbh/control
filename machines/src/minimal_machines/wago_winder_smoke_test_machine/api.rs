use super::WagoWinderSmokeTestMachine;
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
    pub enabled: bool,
    pub target_velocity: i16,
    pub actual_velocity: i16,
    pub target_acceleration: u16,
    pub freq_range_sel: u8,
    pub acc_range_sel: u8,
    pub mode: Option<String>,
    pub ready: bool,
    pub stop2n_ack: bool,
    pub start_ack: bool,
    pub speed_mode_ack: bool,
    pub standstill: bool,
    pub on_speed: bool,
    pub direction_positive: bool,
    pub error: bool,
    pub reset: bool,
    pub position: i64,
    pub raw_position: i64,
    pub di1: bool,
    pub di2: bool,
    pub status_byte1: u8,
    pub status_byte2: u8,
    pub status_byte3: u8,
    pub control_byte1: u8,
    pub control_byte2: u8,
    pub control_byte3: u8,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum WagoWinderSmokeTestMachineEvents {
    State(Event<StateEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    SetStepperEnabled(bool),
    SetStepperVelocity(i16),
    SetStepperPosition(i64),
    SetStepperFreqRange(u8),
    SetStepperAccRange(u8),
    StartCoarseSeek,
    StopByZeroVelocity,
    StopByStop2N,
    ReleaseStop2N,
}

#[derive(Debug, Clone)]
pub struct WagoWinderSmokeTestMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<WagoWinderSmokeTestMachineEvents>
    for WagoWinderSmokeTestMachineNamespace
{
    fn emit(&mut self, events: WagoWinderSmokeTestMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<WagoWinderSmokeTestMachineEvents> for WagoWinderSmokeTestMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            WagoWinderSmokeTestMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

impl MachineApi for WagoWinderSmokeTestMachine {
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetStepperEnabled(enabled) => self.set_stepper_enabled(enabled),
            Mutation::SetStepperVelocity(velocity) => self.set_stepper_velocity(velocity),
            Mutation::SetStepperPosition(position) => self.set_stepper_position(position),
            Mutation::SetStepperFreqRange(factor) => self.set_stepper_freq_range(factor),
            Mutation::SetStepperAccRange(factor) => self.set_stepper_acc_range(factor),
            Mutation::StartCoarseSeek => self.start_coarse_seek(),
            Mutation::StopByZeroVelocity => self.stop_by_zero_velocity(),
            Mutation::StopByStop2N => self.stop_by_stop2n(),
            Mutation::ReleaseStop2N => self.release_stop2n(),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
