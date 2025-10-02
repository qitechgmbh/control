use crate::{MachineApi, MachineMessage};

use super::LaserMachine;
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use control_core_derive::BuildEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    /// diameter measurement in mm
    pub diameter: f64,
    pub x_diameter: Option<f64>,
    pub y_diameter: Option<f64>,
    pub roundness: Option<f64>,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, BuildEvent)]
pub struct StateEvent {
    pub is_default_state: bool,
    /// laser state
    pub laser_state: LaserState,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct LaserState {
    /// higher tolerance in mm
    pub higher_tolerance: f64,
    /// lower tolerance in mm
    pub lower_tolerance: f64,
    /// target diameter in mm
    pub target_diameter: f64,
    /// tolerance bool
    pub in_tolerance: bool,
}

pub enum LaserEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct LaserMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl CacheableEvents<Self> for LaserEvents {
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

#[derive(Deserialize, Serialize)]
/// All values in the Mutation enum should be positive.
/// This ensures that the parameters for setting tolerances and target diameter
/// are valid and meaningful within the context of the LaserMachine's operation.
enum Mutation {
    SetTargetDiameter(f64),
    SetLowerTolerance(f64),
    SetHigherTolerance(f64),
}

impl NamespaceCacheingLogic<LaserEvents> for LaserMachineNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: LaserEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}

impl MachineApi for LaserMachine {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetHigherTolerance(higher_tolerance) => {
                self.set_higher_tolerance(higher_tolerance)
            }
            Mutation::SetLowerTolerance(lower_tolerance) => {
                self.set_lower_tolerance(lower_tolerance);
            }
            Mutation::SetTargetDiameter(target_diameter) => {
                self.set_target_diameter(target_diameter);
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }

    fn api_get_sender(&self) -> smol::channel::Sender<MachineMessage> {
        self.api_sender.clone()
    }
}
