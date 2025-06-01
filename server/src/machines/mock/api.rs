use super::MockMachine;
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, NamespaceInterface,
            cache_duration, cache_one_event,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Mode {
    Standby,
    Running,
}

#[derive(Serialize, Debug, Clone)]
pub struct SineWaveEvent {
    pub amplitude: f64,
}

impl SineWaveEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("SineWaveEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SineWaveStateEvent {
    pub frequency: f64,
}

impl SineWaveStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("MockStateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeStateEvent {
    pub mode: Mode,
}

impl ModeStateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("ModeStateEvent", self.clone())
    }
}

pub enum MockEvents {
    SineWave(Event<SineWaveEvent>),
    SineWaveState(Event<SineWaveStateEvent>),
    ModeState(Event<ModeStateEvent>),
}

#[derive(Debug)]
pub struct MockMachineNamespace(Namespace);

impl MockMachineNamespace {
    pub fn new() -> Self {
        Self(Namespace::new())
    }
}

impl CacheableEvents<MockEvents> for MockEvents {
    fn event_value(&self) -> Result<GenericEvent, serde_json::Error> {
        match self {
            MockEvents::SineWave(event) => event.try_into(),
            MockEvents::SineWaveState(event) => event.try_into(),
            MockEvents::ModeState(event) => event.try_into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_one = cache_one_event();

        match self {
            MockEvents::SineWave(_) => cache_one_hour,
            MockEvents::SineWaveState(_) => cache_one,
            MockEvents::ModeState(_) => cache_one,
        }
    }
}

#[derive(Deserialize, Serialize)]
/// Mutation for controlling the mock machine
enum Mutation {
    /// Set the frequency of the sine wave in millihertz
    SetFrequency(f64),
    SetMode(Mode),
}

impl NamespaceCacheingLogic<MockEvents> for MockMachineNamespace {
    #[instrument(skip_all)]
    fn emit_cached(&mut self, events: MockEvents) {
        let event = match events.event_value() {
            Ok(event) => event,
            Err(err) => {
                tracing::error!("Failed to emit: {:?}", err);
                return;
            }
        };
        let buffer_fn = events.event_cache_fn();
        self.0.emit_cached(&event, &buffer_fn);
    }
}

impl MachineApi for MockMachine {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetFrequency(frequency) => {
                self.set_frequency(frequency);
            }
            Mutation::SetMode(mode) => {
                self.set_mode(mode);
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut dyn NamespaceInterface {
        &mut self.namespace.0
    }
}
