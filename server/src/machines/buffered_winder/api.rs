use std::sync::Arc;

use control_core::{
    machines::api::MachineApi, socketio::{event::{Event, GenericEvent}, namespace::Namespace}
};
use serde::{Deserialize, Serialize};
use smol::channel::Sender;
use socketioxide::extract::SocketRef;

use super::BufferedWinder;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Mode {
    Standby,
    Running,
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

#[derive(Debug)]
pub struct BufferedWinderNamespace {
    pub namespace: Namespace,
}

impl BufferedWinderNamespace {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
        }
    }

}

impl MachineApi for BufferedWinder {
    fn api_mutate(&mut self, value: serde_json::Value) -> Result<(), anyhow::Error> {
        //TODO: implement Mutations
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace.namespace
    }
}

//TODO
