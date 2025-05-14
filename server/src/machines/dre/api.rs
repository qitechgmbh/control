use std::time::Duration;
use crate::serial::devices::dre::DreData;
use super::DreMachine;
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, NamespaceInterface,
            cache_duration,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uom::si::f64::Length;


#[derive(Serialize, Debug, Clone)]
pub struct DiameterEvent {
    pub dre_data: Option<DreData>,
}

impl DiameterEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("DiameterResponseEvent", self.clone())
    }
}

pub enum DreEvents {
    DiameterEvent(Event<DiameterEvent>),
}

#[derive(Debug)]
pub struct DreMachineNamespace(Namespace);

impl DreMachineNamespace {
    pub fn new() -> Self {
        Self(Namespace::new())
    }
}

impl CacheableEvents<DreEvents> for DreEvents {
    fn event_value(&self) -> Result<GenericEvent, serde_json::Error> {
        match self {
            DreEvents::DiameterEvent(event) => event.try_into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_ten_secs = cache_duration(Duration::from_secs(10));

        match self {
            DreEvents::DiameterEvent(_) => cache_ten_secs,
        }
    }
}


#[derive(Deserialize, Serialize)]
enum Mutation {
    TargetSetTargetDiameter(Length),
    TargetSetLowerTolerance(Length),
    TargetSetHigherTolerance(Length)
}

impl NamespaceCacheingLogic<DreEvents> for DreMachineNamespace {
    fn emit_cached(&mut self, events: DreEvents) {
        let event = match events.event_value() {
            Ok(event) => event,
            Err(err) => {
                log::error!(
                    "[{}::emit_cached] Failed to event.event_value(): {:?}",
                    module_path!(),
                    err
                );
                return;
            }
        };
        let buffer_fn = events.event_cache_fn();
        self.0.emit_cached(&event, buffer_fn);
    }
}

impl MachineApi for DreMachine {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::TargetSetHigherTolerance(higher_tolerance)
            =>{self.target_set_higher_tolerance(higher_tolerance)},
            Mutation::TargetSetLowerTolerance(lower_tolerance)
            =>{self.target_set_lower_tolerance(lower_tolerance);},
            Mutation::TargetSetTargetDiameter(target_diameter)
            =>{self.target_set_target_diameter(target_diameter);}
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut dyn NamespaceInterface {
        &mut self.namespace.0
    }
}
