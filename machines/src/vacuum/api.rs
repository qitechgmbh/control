use super::VacuumMachine;
use crate::{MachineApi, MachineMessage, vacuum::Mode};
use anyhow::anyhow;
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
pub struct StateEvent 
{
    pub mode: u8,
    pub interval_time_off: f64,
    pub interval_time_on:  f64,
}

impl StateEvent 
{
    pub fn build(&self) -> Event<Self> 
    {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct LiveValuesEvent 
{
    
}

impl LiveValuesEvent 
{
    pub fn build(&self) -> Event<Self> 
    {
        Event::new("LiveValuesEvent", self.clone())
    }
}

pub enum VacuumMachineEvents 
{
    State(Event<StateEvent>),
    LiveValues(Event<LiveValuesEvent>),
}

#[derive(Deserialize, Serialize)]
pub enum Mutation 
{
    SetMode(Mode),
    SetIntervalTimeOff(f64),
    SetIntervalTimeOn(f64),
}

#[derive(Debug, Clone)]
pub struct VacuumMachineNamespace 
{
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<VacuumMachineEvents> for VacuumMachineNamespace 
{
    fn emit(&mut self, events: VacuumMachineEvents) 
    {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace 
        {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<VacuumMachineEvents> for VacuumMachineEvents 
{
    fn event_value(&self) -> GenericEvent 
    {
        match self 
        {
            VacuumMachineEvents::State(event)      => event.clone().into(),
            VacuumMachineEvents::LiveValues(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn 
    {
        cache_first_and_last_event()
    }
}

impl MachineApi for VacuumMachine 
{
    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> 
    {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> 
    {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        
        match mutation 
        {
            Mutation::SetMode(value)           => self.set_mode(value),
            Mutation::SetIntervalTimeOff(value) => self.set_interval_time_off(value),
            Mutation::SetIntervalTimeOn(value)  => self.set_interval_time_on(value),
        }

        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> 
    {
        self.namespace.namespace.clone()
    }
}
