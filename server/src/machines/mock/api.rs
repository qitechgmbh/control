use super::MockMachine;
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Standby,
    Running,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub amplitude_sum: f64,
    pub amplitude1: f64,
    pub amplitude2: f64,
    pub amplitude3: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, BuildEvent)]
pub struct StateEvent {
    pub is_default_state: bool,
    /// sine wave frequencies in millihertz
    pub frequency1: f64,
    pub frequency2: f64,
    pub frequency3: f64,
    /// mode state
    pub mode_state: ModeState,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ModeState {
    /// current mode
    pub mode: Mode,
}

pub enum MockEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct MockMachineNamespace {
    pub namespace: Arc<Mutex<Namespace>>,
}

impl CacheableEvents<Self> for MockEvents {
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
/// Mutation for controlling the mock machine
enum Mutation {
    /// Set the frequency of the sine wave in millihertz
    SetFrequency1(f64),
    SetFrequency2(f64),
    SetFrequency3(f64),
    SetMode(Mode),
}

impl NamespaceCacheingLogic<MockEvents> for MockMachineNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: MockEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();

        let mut namespace = self.namespace.lock_blocking();
        namespace.emit(event, &buffer_fn);
    }
}

impl MachineApi for MockMachine {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetFrequency1(frequency) => {
                self.set_frequency1(frequency);
            }
            Mutation::SetFrequency2(frequency) => {
                self.set_frequency2(frequency);
            }
            Mutation::SetFrequency3(frequency) => {
                self.set_frequency3(frequency);
            }
            Mutation::SetMode(mode) => {
                self.set_mode(mode);
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>> {
        self.namespace.namespace.clone()
    }
}
