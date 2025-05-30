use super::LaserMachine;
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_duration,
            cache_one_event,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use tracing::instrument;

#[derive(Serialize, Debug, Clone)]
pub struct DiameterEvent {
    pub diameter: f64,
}

impl DiameterEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("DiameterEvent", self.clone())
    }
}
#[derive(Serialize, Debug, Clone)]
pub struct LaserStateEvent {
    pub higher_tolerance: f64,
    pub lower_tolerance: f64,
    pub target_diameter: f64,
}

impl LaserStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LaserStateEvent", self.clone())
    }
}

pub enum LaserEvents {
    Diameter(Event<DiameterEvent>),
    LaserState(Event<LaserStateEvent>),
}

#[derive(Debug)]
pub struct LaserMachineNamespace(Namespace);

impl LaserMachineNamespace {
    pub fn new() -> Self {
        Self(Namespace::new())
    }
}

impl CacheableEvents<LaserEvents> for LaserEvents {
    fn event_value(&self) -> Result<GenericEvent, serde_json::Error> {
        match self {
            LaserEvents::Diameter(event) => event.try_into(),
            LaserEvents::LaserState(event) => event.try_into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            LaserEvents::Diameter(_) => cache_one_hour,
            LaserEvents::LaserState(_) => cache_one,
        }
    }
}

#[derive(Deserialize, Serialize)]
/// All values in the Mutation enum should be positive.
/// This ensures that the parameters for setting tolerances and target diameter
/// are valid and meaningful within the context of the LaserMachine's operation.
enum Mutation {
    TargetSetTargetDiameter(f64),
    TargetSetLowerTolerance(f64),
    TargetSetHigherTolerance(f64),
}

impl NamespaceCacheingLogic<LaserEvents> for LaserMachineNamespace {
    #[instrument(skip_all)]
    fn emit_cached(&mut self, events: LaserEvents) {
        let event = match events.event_value() {
            Ok(event) => event,
            Err(err) => {
                tracing::error!("Failed to emit: {:?}", err);
                return;
            }
        };
        let event = Arc::new(event);
        let buffer_fn = events.event_cache_fn();
        self.0.emit_cached(event, &buffer_fn);
    }
}

impl MachineApi for LaserMachine {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::TargetSetHigherTolerance(higher_tolerance) => {
                self.target_set_higher_tolerance(higher_tolerance)
            }
            Mutation::TargetSetLowerTolerance(lower_tolerance) => {
                self.target_set_lower_tolerance(lower_tolerance);
            }
            Mutation::TargetSetTargetDiameter(target_diameter) => {
                self.target_set_target_diameter(target_diameter);
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.0
    }
}
