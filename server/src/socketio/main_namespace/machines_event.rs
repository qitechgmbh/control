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

    pub fn build(&self, app_state: Arc<AppState>) -> Event<MachinesEvent> {
        let machine_objs = app_state.get_machine_objs();
        Event::new(
            Self::NAME,
            MachinesEvent {
                machines: machine_objs,
            },
        )
    }
}
