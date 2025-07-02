use std::sync::Arc;

use control_core::{
    machines::api::MachineApi, rest::mutation, socketio::{event::{Event, GenericEvent}, namespace::Namespace}
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;

use super::Buffer1;

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

impl Buffer1Namespace {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            namespace: Namespace::new(socket_queue_tx),
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