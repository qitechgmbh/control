use crate::machines::mock2::{self, ConnectedMachine, ConnectedMachineData};

use super::Mock2Machine;
use control_core::{
    machines::{api::MachineApi, identification::MachineIdentificationUnique},
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            cache_duration, cache_first_and_last_event, CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use std::{sync::Arc, time::Duration};
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Mode {
    Standby,
    Running,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    /// sine wave amplitude value
    pub sine_wave_amplitude: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StateEvent {
    pub is_default_state: bool,
    /// sine wave state
    pub sine_wave_state: SineWaveState,
    /// mode state
    pub mode_state: ModeState,
    /// connected machine state
    pub connected_machine_state: ConnectedMachineState,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct SineWaveState {
    /// frequency in millihertz
    pub frequency: f64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ModeState {
    /// current mode
    pub mode: Mode,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ConnectedMachineState {
    pub machine_identification_unique: Option<MachineIdentificationUnique>,
    pub is_available: bool,
}

pub enum Mock2Events {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct Mock2MachineNamespace {
    pub namespace: Namespace,
}

impl Mock2MachineNamespace {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
        }
    }
}

impl CacheableEvents<Mock2Events> for Mock2Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            Mock2Events::LiveValues(event) => event.into(),
            Mock2Events::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_first_and_last = cache_first_and_last_event();

        match self {
            Mock2Events::LiveValues(_) => cache_one_hour,
            Mock2Events::State(_) => cache_first_and_last,
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

impl NamespaceCacheingLogic<Mock2Events> for Mock2MachineNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: Mock2Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        self.namespace.emit(event, &buffer_fn);
    }
}

impl MachineApi for Mock2Machine {
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

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.namespace
    }
}
