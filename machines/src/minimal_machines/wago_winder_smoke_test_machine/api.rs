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
pub struct AxisState {
    pub enabled: bool,
    pub target_velocity: i16,
    pub target_acceleration: u16,
    pub freq_range_sel: u8,
    pub acc_range_sel: u8,
    pub mode: Option<String>,
    pub speed_mode_ack: bool,
    pub di1: bool,
    pub di2: bool,
    pub status_byte1: u8,
    pub status_byte2: u8,
    pub status_byte3: u8,
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub axes: [AxisState; 1],
    pub digital_output1: bool,
    pub digital_output2: bool,
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
    SetStepperEnabled { axis: usize, enabled: bool },
    SetStepperVelocity { axis: usize, velocity: i16 },
    SetStepperFreqRange { axis: usize, factor: u8 },
    SetStepperAccRange { axis: usize, factor: u8 },
    SetDigitalOutput { port: usize, value: bool },
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
            Mutation::SetStepperEnabled { axis, enabled } => {
                self.set_stepper_enabled(axis, enabled)
            }
            Mutation::SetStepperVelocity { axis, velocity } => {
                self.set_stepper_velocity(axis, velocity)
            }
            Mutation::SetStepperFreqRange { axis, factor } => {
                self.set_stepper_freq_range(axis, factor)
            }
            Mutation::SetStepperAccRange { axis, factor } => {
                self.set_stepper_acc_range(axis, factor)
            }
            Mutation::SetDigitalOutput { port, value } => self.set_digital_output(port, value),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
