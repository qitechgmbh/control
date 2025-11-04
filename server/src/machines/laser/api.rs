use super::LaserMachine;
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_duration,
            cache_first_and_last_event,
        },
    },
};
use control_core_derive::BuildEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::lock::Mutex;
use std::{sync::Arc, time::Duration};
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
}

pub enum LaserEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct LaserMachineNamespace {
    pub namespace: Arc<Mutex<Namespace>>,
}

impl CacheableEvents<Self> for LaserEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_first_and_last = cache_first_and_last_event();

        match self {
            Self::LiveValues(_) => cache_one_hour,
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

        let mut namespace = self.namespace.lock_blocking();
        namespace.emit(event, &buffer_fn);
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

    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>> {
        self.namespace.namespace.clone()
    }

    fn api_event(&mut self, events: Option<&control_core::rest::mutation::EventFields>) -> Result<Value, anyhow::Error> {
        use uom::si::length::millimeter;

        let live_values = LiveValuesEvent {
            diameter: self.diameter.get::<millimeter>(),
            x_diameter: self.x_diameter.map(|d| d.get::<millimeter>()),
            y_diameter: self.y_diameter.map(|d| d.get::<millimeter>()),
            roundness: self.roundness,
        };

        let state = self.build_state_event();

        // Build response with requested events and fields
        let mut result = serde_json::Map::new();

        // Determine which events to include
        let (include_live_values, live_values_fields) = match events {
            None => (true, None),
            Some(ef) => (ef.live_values.is_some(), ef.live_values.as_ref()),
        };

        let (include_state, state_fields) = match events {
            None => (true, None),
            Some(ef) => (ef.state.is_some(), ef.state.as_ref()),
        };

        // Add LiveValues if requested
        if include_live_values {
            let live_values_json = serde_json::to_value(live_values)?;
            let filtered = crate::rest::event_filter::filter_event_fields(live_values_json, live_values_fields)?;
            if !filtered.is_null() {
                result.insert("LiveValues".to_string(), filtered);
            }
        }

        // Add State if requested
        if include_state {
            let state_json = serde_json::to_value(state)?;
            let filtered = crate::rest::event_filter::filter_event_fields(state_json, state_fields)?;
            if !filtered.is_null() {
                result.insert("State".to_string(), filtered);
            }
        }

        Ok(Value::Object(result))
    }
}