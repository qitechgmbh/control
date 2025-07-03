use std::{sync::Arc, time::Duration};

use super::Buffer1;
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            cache_duration, cache_one_event, CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use tracing::instrument;

pub enum Buffer1Events {
    Mode(Event<ModeStateEvent>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Mode {
    Standby,
    Running,
    FillingBuffer,
    EmptyingBuffer,
}

#[derive(Serialize, Debug, Clone)]
pub struct ModeStateEvent {
    pub mode: Mode,
}

impl ModeStateEvent {
    pub fn build(&self) ->  Event<Self> {
        Event::new("ModeStateEvent", self.clone())
    }
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    BufferGoUp,
    BufferGoDown,
}

#[derive(Debug)]
pub struct Buffer1Namespace {
    pub namespace: Namespace,
}

impl NamespaceCacheingLogic<Buffer1Events> for Buffer1Namespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: Buffer1Events) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        self.namespace.emit(event, &buffer_fn);
    }
}

impl Buffer1Namespace {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
        }
    }
}

impl CacheableEvents<Buffer1Events> for Buffer1Events {
    fn event_value(&self) -> GenericEvent {
        match self {
            Buffer1Events::Mode(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1));
        let cache_one = cache_one_event();
        
        match self {
            Buffer1Events::Mode(_) => cache_one_hour,
        }
    }
}

impl MachineApi for Buffer1 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::BufferGoUp => self.buffer_go_up(),
            Mutation::BufferGoDown => self.buffer_go_down(),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.namespace
    }
}