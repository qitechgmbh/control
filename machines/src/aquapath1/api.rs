use super::{AquaPathV1, AquaPathV1Mode};
use crate::{MachineApi, MachineMessage};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use std::sync::Arc;
use tracing::instrument;

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub front_flow: f64,
    pub back_flow: f64,
    pub front_temperature: f64,
    pub back_temperature: f64,
    pub front_temp_reservoir: f64,
    pub back_temp_reservoir: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub is_default_state: bool,
    /// mode state
    pub mode_state: ModeState,
    pub flow_states: FlowStates,
    pub temperature_states: TempStates,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct TempStates {
    pub front: TempState,
    pub back: TempState,
}

#[derive(Serialize, Debug, Clone)]
pub struct TempState {
    pub temperature: f64,
    pub target_temperature: f64,
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeState {
    pub mode: AquaPathV1Mode,
}
#[derive(Serialize, Debug, Clone)]
pub struct FlowStates {
    pub front: FlowState,
    pub back: FlowState,
}
#[derive(Serialize, Debug, Clone)]
pub struct FlowState {
    pub flow: f64,
    pub should_flow: bool,
}

pub enum AquaPathV1Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    //Mode
    SetAquaPathMode(AquaPathV1Mode),

    SetFrontTemperature(f64),
    SetBackTemperature(f64),

    SetFrontFlow(bool),
    SetBackFlow(bool),
}

#[derive(Debug, Clone)]
pub struct AquaPathV1Namespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<AquaPathV1Events> for AquaPathV1Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: AquaPathV1Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}

impl CacheableEvents<AquaPathV1Events> for AquaPathV1Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            AquaPathV1Events::LiveValues(event) => event.into(),
            AquaPathV1Events::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_first_and_last = cache_first_and_last_event();

        match self {
            AquaPathV1Events::LiveValues(_) => cache_first_and_last,
            AquaPathV1Events::State(_) => cache_first_and_last,
        }
    }
}

impl MachineApi for AquaPathV1 {
    fn api_get_sender(&self) -> Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::SetAquaPathMode(mode) => self.set_mode_state(mode),
            Mutation::SetBackTemperature(temperature) => {
                self.set_target_temperature(temperature, super::AquaPathSideType::Back)
            }

            Mutation::SetFrontTemperature(temperature) => {
                self.set_target_temperature(temperature, super::AquaPathSideType::Front)
            }

            Mutation::SetBackFlow(should_pump) => {
                self.set_should_pump(should_pump, super::AquaPathSideType::Back)
            }
            Mutation::SetFrontFlow(should_pump) => {
                self.set_should_pump(should_pump, super::AquaPathSideType::Front)
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
