use std::time::Instant;

use self::api::{StateEvent, Wago750_460MachineEvents, Wago750_460MachineNamespace};
use crate::{
    MachineMessage, VENDOR_QITECH, WAGO_750_460_MACHINE,
    machine_identification::MachineIdentification,
};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::io::temperature_input::TemperatureInputInput,
    machines::MachineIdentificationUnique,
};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_460Machine {
    // --- mandatory plumbing -------------------------------------------------
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: Wago750_460MachineNamespace,
    pub last_state_emit: Instant,

    // --- hardware -----------------------------------------------------------
    pub temperature_inputs: [TemperatureInputInput; 4],
}

impl Wago750_460Machine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_750_460_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        let mut temperatures: [Option<f32>; 4] = [None; 4];
        let mut errors = [false; 4];
        for (i, ti) in self.temperature_inputs.iter().enumerate() {
            errors[i] = ti.error;
            if !ti.error {
                temperatures[i] = Some(ti.temperature);
            }
        }
        StateEvent {
            temperatures,
            errors,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(Wago750_460MachineEvents::State(event));
    }
}
