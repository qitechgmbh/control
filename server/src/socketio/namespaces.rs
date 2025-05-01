use control_core::socketio::{namespace::NamespaceInterface, namespace_id::NamespaceId};

use crate::app_state::APP_STATE;

use super::main_namespace::MainRoom;

pub struct Namespaces {
    pub main_namespace: MainRoom,
}

impl Namespaces {
    pub fn new() -> Self {
        Self {
            main_namespace: MainRoom::new(),
        }
    }

    pub async fn apply_mut(
        &mut self,
        namespace_id: NamespaceId,
        callback: impl FnOnce(Result<&mut dyn NamespaceInterface, anyhow::Error>),
    ) {
        match namespace_id {
            NamespaceId::Main => callback(Ok(&mut self.main_namespace.0)),
            NamespaceId::Machine(machine_identification_unique) => {
                let mut machines_guard = APP_STATE.machines.write().await;

                // get machine
                let machine = match machines_guard.get_mut(&machine_identification_unique) {
                    Some(machine) => machine,
                    None => {
                        callback(Err(anyhow::anyhow!(
                            "Machine {} not found",
                            machine_identification_unique
                        )));
                        return;
                    }
                };

                // check if machine has error
                let machine = match machine {
                    Ok(machine) => machine,
                    Err(err) => {
                        callback(Err(anyhow::anyhow!(
                            "Machine {} has error: {}",
                            machine_identification_unique,
                            err
                        )));
                        return;
                    }
                };

                let namespace = machine.api_event_namespace();
                callback(Ok(namespace));
            }
        }
    }
}
