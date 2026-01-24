use crate::{MachineApi, MachineMessage};

use super::Pelletizer;
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use control_core_derive::BuildEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use std::sync::Arc;
use tracing::instrument;

#[derive(Serialize, Debug, Clone)]
pub struct LiveValuesEvent 
{
    pub inverter_values: InverterLiveValues
}

#[derive(Serialize, Debug, Clone)]
pub struct InverterLiveValues 
{
    pub frequency:   f64,
    pub temperature: f64,
    pub voltage:     f64,
    pub current:     f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, BuildEvent)]
pub struct StateEvent {
    pub is_default_state: bool,
    
    pub inverter_state: InverterState,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct InverterState 
{
    pub running_state:      u8,
    pub frequency_target:   u16,
    pub acceleration_level: u8,
    pub deceleration_level: u8,
    pub error_code:         u16,
    
    pub system_status:      u16,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct RotationState 
{
    pub forward: bool,
}

pub enum PelletMachineEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct PelletMachineNamespace {
    pub namespace: Option<Namespace>,
}

#[derive(Debug, Deserialize, Serialize)]
/// Mutation for controlling the Pellet machine
enum Mutation 
{
    SetRunState(u8),
    SetFrequencyTarget(u8),
    SetAccelerationLevel(u8),
    SetDecelerationLevel(u8),
}

impl MachineApi for Pelletizer {
    fn api_get_sender(&self) -> Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> 
    {
        let mutation: Mutation = serde_json::from_value(request_body)?;

        tracing::error!("Received: {:?}", &mutation);

        match mutation 
        {
            Mutation::SetRunState(state) => 
            {
                // self.set_run_state(state);
            }
            Mutation::SetFrequencyTarget(frequency) => 
            {
                self.set_frequency(frequency as u16 * 10);
            }
            Mutation::SetAccelerationLevel(speed) => 
            {
                // self.set_speed(speed);
            }
            Mutation::SetDecelerationLevel(speed) => 
            {
                // self.set_speed(speed);
            }
        }
        
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}

//TODO; rename NamespaceCacheingLogic to NamespaceCachingLogic
impl NamespaceCacheingLogic<PelletMachineEvents> for PelletMachineNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: PelletMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        if let Some(ns) = &mut self.namespace { ns.emit(event, &buffer_fn) }
    }
}

impl CacheableEvents<Self> for PelletMachineEvents {
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