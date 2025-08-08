use std::sync::Arc;

use control_core::socketio::{
    event::GenericEvent, namespace::Namespace, namespace_id::NamespaceId,
};
use smol::channel::Sender;
use socketioxide::extract::SocketRef;

use crate::app_state;

use super::main_namespace::MainRoom;

pub struct Namespaces {
    pub main_namespace: MainRoom,
}

impl Namespaces {
    pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self {
        Self {
            main_namespace: MainRoom::new(socket_queue_tx),
        }
    }

    pub async fn apply_mut(
        &mut self,
        namespace_id: NamespaceId,
        app_state: &Arc<app_state::AppState>,
        callback: impl FnOnce(Result<&mut Namespace, anyhow::Error>),
    ) {
        match namespace_id {
            NamespaceId::Main => callback(Ok(&mut self.main_namespace.namespace)),
            NamespaceId::Machine(machine_identification_unique) => {
                // Lock machines and work directly with the reference to avoid cloning issues
                let machines_guard = app_state.machines.read().await;

                // get machine
                let machine = match machines_guard.get(&machine_identification_unique) {
                    Some(machine) => machine,
                    None => {
                        callback(Err(anyhow::anyhow!(
                            "[{}::Namespaces::appply_mut] Machine {:?} not found",
                            module_path!(),
                            machine_identification_unique
                        )));
                        return;
                    }
                };

                // Ok(machine) => machine,
                // err => {
                //     callback(Err(anyhow::anyhow!(
                //         "[{}::Namespaces::appply_mut] Machine {:?} has error: {}",
                //         module_path!(),
                //         machine_identification_unique,
                //         err
                //     )));
                //     return;
                // }

                // check if machine has error
                let machine = match machine.machine_connection {
                    control_core::machines::manager::MachineConnection::Error(error) => {
                        callback(Err(anyhow::anyhow!(
                            "[{}::Namespaces::appply_mut] Machine {:?} has error: {}",
                            module_path!(),
                            machine_identification_unique,
                            error
                        )));
                        return;
                    }
                    control_core::machines::manager::MachineConnection::Disconnected => {
                        callback(Err(anyhow::anyhow!(
                            "[{}::Namespaces::appply_mut] Machine {:?} has disconnected",
                            module_path!(),
                            machine_identification_unique
                        )));
                        return;
                    }
                    control_core::machines::manager::MachineConnection::Connected(mutex) => mutex,
                };
                let mut machine_guard = machine.lock().await;
                let namespace = machine_guard.api_event_namespace();
                callback(Ok(namespace));
            }
        }
    }
}
