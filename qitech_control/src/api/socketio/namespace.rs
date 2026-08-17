use std::collections::HashMap;

use serde::Serialize;
use socketioxide::extract::SocketRef;

use crate::api::socketio::namespace_id::NamespaceId;

#[derive(Debug, Clone)]
pub struct Namespaces {
    registry: HashMap<NamespaceId, Vec<SocketRef>>,
}



#[derive(Debug, Clone)]
pub struct Namespace {
    pub sockets: Vec<SocketRef>,
}

impl Namespace {
    pub fn new() -> Self {
        Self { sockets: vec![] }
    }
}

impl Namespace {
    pub fn push(&mut self, socket: SocketRef) {
        self.sockets.push(socket);
    }

    pub fn remove(&mut self, socket: SocketRef) {
        self.sockets.retain(|s| s.id != socket.id);
    }

    pub fn disconnect_all(&mut self) {
        for socket in self.sockets.drain(..) {
            let _ = socket.disconnect();
        }
    }

    pub fn emit<T: Serialize>(&mut self, event: &Event<T>) {
        for socket in &self.sockets {
            // TODO: use error ?
            _ = socket.emit("event", event);
        }
    }
}

#[derive(Serialize)]
pub struct Event<T: Serialize> {
    pub name: String,
    pub data: T,
    /// Timestamp in milliseconds
    pub ts: u64,
}
