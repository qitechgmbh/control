use std::sync::Arc;

use control_core::{
    machines::api::MachineApi, socketio::{event::GenericEvent, namespace::Namespace}
};
use smol::channel::Sender;
use socketioxide::extract::SocketRef;

use super::BufferedWinder;

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
