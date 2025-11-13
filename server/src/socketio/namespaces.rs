use super::{main_namespace::MainRoom, namespace_id::NamespaceId};
use control_core::socketio::event::GenericEvent;
use smol::channel::Sender;
use socketioxide::extract::SocketRef;
use std::{collections::HashMap, sync::Arc};

pub struct Namespaces {
    pub main_namespace: MainRoom,
    pub machine_namespaces: HashMap<NamespaceId, control_core::socketio::namespace::Namespace>,
}

impl Namespaces {
    pub async fn apply_mut(
        &mut self,
        namespace_id: NamespaceId,
    ) -> Result<&mut control_core::socketio::namespace::Namespace, anyhow::Error> {
        match namespace_id.clone() {
            NamespaceId::Main => Ok(&mut self.main_namespace.namespace),
            NamespaceId::Machine(_) => {
                let res = self.machine_namespaces.get_mut(&namespace_id);
                let namespace = match res {
                    Some(namespace) => namespace,
                    None => return Err(anyhow::anyhow!("Namespace not found")),
                };
                Ok(namespace)
            }
        }
    }

    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            main_namespace: MainRoom::new(socket_queue_tx),
            machine_namespaces: HashMap::new(),
        }
    }
}
