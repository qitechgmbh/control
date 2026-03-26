use machine_implementations::machine_identification::QiTechMachineIdentificationUnique;
use serde::{Deserialize, Serialize};
use crate::apis::socketio::main_namespace::Event;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachinesEvent {
    pub machines: Vec<MachineObj>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineObj {
    pub machine_identification_unique: QiTechMachineIdentificationUnique,
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
