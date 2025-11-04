use std::{sync::Arc, time::Duration};

use super::{BufferV1, BufferV1Mode};
use control_core::{
    machines::{
        api::MachineApi, connection::MachineCrossConnectionState,
        identification::MachineIdentificationUnique,
    },
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
use smol::lock::Mutex;
use tracing::instrument;

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    /// mode state
    pub mode_state: ModeState,
    /// connected machine state
    pub connected_machine_state: MachineCrossConnectionState,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum BufferV1Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeState {
    pub mode: BufferV1Mode,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Standby,
    FillingBuffer,
    EmptyingBuffer,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ConnectedMachineState {
    /// Connected Machine
    pub machine_identification_unique: Option<MachineIdentificationUnique>,
    pub is_available: bool,
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    // Mode
    SetBufferMode(BufferV1Mode),

    // Connected Machine
    SetConnectedMachine(MachineIdentificationUnique),

    // Disconnect Machine
    DisconnectMachine(MachineIdentificationUnique),
}

#[derive(Debug)]
pub struct Buffer1Namespace {
    pub namespace: Arc<Mutex<Namespace>>,
}

impl NamespaceCacheingLogic<BufferV1Events> for Buffer1Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: BufferV1Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        let mut namespace = self.namespace.lock_blocking();
        namespace.emit(event, &buffer_fn);
    }
}

impl CacheableEvents<Self> for BufferV1Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            Self::LiveValues(_) => cache_one_hour,
            Self::State(_) => cache_one,
        }
    }
}

impl MachineApi for BufferV1 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetBufferMode(mode) => self.set_mode_state(mode),
            Mutation::SetConnectedMachine(machine_identification_unique) => {
                self.set_connected_winder(machine_identification_unique);
            }
            Mutation::DisconnectMachine(machine_identification_unique) => {
                self.disconnect_winder(machine_identification_unique);
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>> {
        self.namespace.namespace.clone()
    }

    fn api_event(&mut self, events: Option<&control_core::rest::mutation::EventFields>) -> Result<Value, anyhow::Error> {
        let live_values = LiveValuesEvent {};

        let state = StateEvent {
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            connected_machine_state: self.connected_winder.to_state(),
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