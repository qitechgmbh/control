use super::DrywellMachine;
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
use tracing::instrument;

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub status: u16,
    pub temp_process: f64,
    pub temp_safety: f64,
    pub temp_regen_in: f64,
    pub temp_regen_out: f64,
    pub temp_fan_inlet: f64,
    pub temp_return_air: f64,
    pub temp_dew_point: f64,
    pub pwm_fan1: f64,
    pub pwm_fan2: f64,
    pub power_process: f64,
    pub power_regen: f64,
    pub alarm: u16,
    pub warning: u16,
    pub target_temperature: f64,
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    SetStartStop(bool),
    SetTargetTemperature(f64),
}

impl MachineApi for DrywellMachine {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetStartStop(_) => self.set_start_stop(),
            Mutation::SetTargetTemperature(temp) => self.set_target_temperature(temp),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<control_core::socketio::namespace::Namespace> {
        self.namespace.namespace.clone()
    }

    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub is_default_state: bool,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum DrywellEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct DrywellMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl CacheableEvents<Self> for DrywellEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_first_and_last = cache_first_and_last_event();
        match self {
            Self::LiveValues(_) => cache_first_and_last,
            Self::State(_) => cache_first_and_last,
        }
    }
}

impl NamespaceCacheingLogic<DrywellEvents> for DrywellMachineNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: DrywellEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}
