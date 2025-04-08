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
                let ethercat_setup_guard = APP_STATE.ethercat_setup.read().await;
                let ethercat_setup_guard = match ethercat_setup_guard.as_ref() {
                    Some(ethercat_setup_guard) => ethercat_setup_guard,
                    None => {
                        callback(Err(anyhow::anyhow!("Ethercat setup not found")));
                        return;
                    }
                };

                // get machine
                let machine = match ethercat_setup_guard
                    .machines
                    .get(&machine_identification_unique)
                {
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

                let mut machine_guard = machine.write().await;
                let namespace = machine_guard.api_event_namespace();
                callback(Ok(namespace));
            }
        }
    }
}
