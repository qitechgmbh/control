use control_core::socketio::event::Event;
use machines::machine_identification::MachineIdentificationUnique;
use serde::{Deserialize, Serialize};

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

    pub fn build(&self, machine_objs: Vec<MachineObj>) -> Event<MachinesEvent> {
        Event::new(
            Self::NAME,
            MachinesEvent {
                machines: machine_objs,
            },
        )
    }
}
