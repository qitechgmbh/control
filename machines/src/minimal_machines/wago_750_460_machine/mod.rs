use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::temperature_input::{TemperatureInput, TemperatureInputError};
use smol::channel::{Receiver, Sender};

use self::api::{StateEvent, Wago750_460MachineEvents, Wago750_460MachineNamespace};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_750_460_MACHINE,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_460Machine {
    // --- mandatory plumbing -------------------------------------------------
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: Wago750_460MachineNamespace,
    pub last_state_emit: Instant,

    // --- hardware -----------------------------------------------------------
    pub temperature_inputs: [TemperatureInput; 4],
}

impl Machine for Wago750_460Machine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl Wago750_460Machine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_750_460_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        let mut temperatures = [None; 4];
        let mut errors = [false; 4];
        for (i, ti) in self.temperature_inputs.iter().enumerate() {
            match ti.get_temperature() {
                Ok(t) => temperatures[i] = Some(t),
                // WireBreak is not currently returned by the 750-460 driver,
                // but we handle it defensively for forward-compatibility.
                Err(TemperatureInputError::WireBreak)
                | Err(TemperatureInputError::OverVoltage)
                | Err(TemperatureInputError::UnderVoltage) => errors[i] = true,
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
