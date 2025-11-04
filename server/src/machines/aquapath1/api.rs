use std::{sync::Arc, time::Duration};

use super::{AquaPathV1, AquaPathV1Mode};
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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::lock::Mutex;
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

#[derive(Debug)]
pub struct AquaPathV1Namespace {
    pub namespace: Arc<Mutex<Namespace>>,
}

impl NamespaceCacheingLogic<AquaPathV1Events> for AquaPathV1Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: AquaPathV1Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        let mut namespace = self.namespace.lock_blocking();
        namespace.emit(event, &buffer_fn);
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
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_first_and_last = cache_first_and_last_event();

        match self {
            AquaPathV1Events::LiveValues(_) => cache_one_hour,
            AquaPathV1Events::State(_) => cache_first_and_last,
        }
    }
}

impl MachineApi for AquaPathV1 {
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

    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>> {
        self.namespace.namespace.clone()
    }

    fn api_event(
        &mut self,
        events: Option<&control_core::rest::mutation::EventFields>,
    ) -> Result<Value, anyhow::Error> {
        use uom::si::thermodynamic_temperature::degree_celsius;
        use uom::si::volume_rate::liter_per_minute;

        let live_values = LiveValuesEvent {
            front_flow: self.front_controller.current_flow.get::<liter_per_minute>(),
            back_flow: self.back_controller.current_flow.get::<liter_per_minute>(),
            front_temperature: self
                .front_controller
                .current_temperature
                .get::<degree_celsius>(),
            back_temperature: self
                .back_controller
                .current_temperature
                .get::<degree_celsius>(),
            front_temp_reservoir: self.front_controller.temp_reservoir.get::<degree_celsius>(),
            back_temp_reservoir: self.back_controller.temp_reservoir.get::<degree_celsius>(),
        };

        let state = StateEvent {
            is_default_state: false,
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            temperature_states: TempStates {
                front: TempState {
                    temperature: self
                        .front_controller
                        .current_temperature
                        .get::<degree_celsius>(),
                    target_temperature: self
                        .front_controller
                        .target_temperature
                        .get::<degree_celsius>(),
                },
                back: TempState {
                    temperature: self
                        .back_controller
                        .current_temperature
                        .get::<degree_celsius>(),
                    target_temperature: self
                        .back_controller
                        .target_temperature
                        .get::<degree_celsius>(),
                },
            },
            flow_states: FlowStates {
                front: FlowState {
                    flow: self.front_controller.current_flow.get::<liter_per_minute>(),
                    should_flow: self.front_controller.should_pump,
                },
                back: FlowState {
                    flow: self.back_controller.current_flow.get::<liter_per_minute>(),
                    should_flow: self.back_controller.should_pump,
                },
            },
        };

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
            let filtered = crate::rest::event_filter::filter_event_fields(
                live_values_json,
                live_values_fields,
            )?;
            if !filtered.is_null() {
                result.insert("LiveValues".to_string(), filtered);
            }
        }

        // Add State if requested
        if include_state {
            let state_json = serde_json::to_value(state)?;
            let filtered =
                crate::rest::event_filter::filter_event_fields(state_json, state_fields)?;
            if !filtered.is_null() {
                result.insert("State".to_string(), filtered);
            }
        }

        Ok(Value::Object(result))
    }
}
