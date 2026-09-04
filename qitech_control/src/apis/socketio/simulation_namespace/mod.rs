//! Socket.io namespace for the live extruder heating simulation, `/simulation`.
//!
//! A fixed, singleton namespace — not one-per-connected-client like
//! `/machine/{vendor}/{machine}/{serial}` — modelled directly on
//! [`super::main_namespace::MainRoom`], since there is exactly one simulation
//! for the whole backend, the same way there is exactly one `/main`.

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use socketioxide::extract::SocketRef;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::instrument;

pub mod events;

pub use events::{LiveValuesEvent, PerZone, StateEvent};

pub struct SimulationRoom {
    pub namespace: Namespace,
}

impl SimulationRoom {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
        }
    }
}

pub enum SimulationEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

impl NamespaceCacheingLogic<SimulationEvents> for SimulationRoom
where
    SimulationEvents: CacheableEvents<SimulationEvents>,
{
    #[instrument(skip_all)]
    fn emit(&mut self, event: SimulationEvents) {
        let buffer_fn = event.event_cache_fn();
        let generic_event = Arc::new(event.event_value());
        self.namespace.emit(generic_event, &buffer_fn);
    }
}

impl CacheableEvents<Self> for SimulationEvents {
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
