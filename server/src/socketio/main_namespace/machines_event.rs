use std::sync::Arc;

use control_core::{machines::identification::MachineIdentificationUnique, socketio::event::Event};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachinesEvent {
    pub machines: Vec<MachineObj>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineObj {
    pub machine_identification_unique: MachineIdentificationUnique,
    pub error: Option<String>,
}
pub struct MachinesEventBuilder();

impl MachinesEventBuilder {
    const NAME: &'static str = "MachinesEvent";

    pub async fn build(&self, app_state: Arc<AppState>) -> Event<MachinesEvent> {
        let mut machine_objs: Vec<_> = vec![];
        // TODO: some filter map action
        let machines_guard = app_state.machines.read().await;
        for machine in machines_guard.iter() {
            let slot = machine.1.lock_blocking();
            let connection = &slot.machine_connection;
            machine_objs.push(MachineObj {
                machine_identification_unique: machine.0.clone(),
                error: connection.to_error().map(|e| e.to_string()),
            });
        }

        Event::new(
            Self::NAME,
            MachinesEvent {
                machines: machine_objs,
            },
        )
    }
}
