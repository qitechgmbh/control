use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_input::DigitalInput;
use smol::channel::{Receiver, Sender};

use self::api::{StateEvent, Wago750_430UfmEvents, Wago750_430UfmNamespace};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_750_430_UFM,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_430UfmMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: Wago750_430UfmNamespace,
    pub last_state_emit: Instant,
    pub inputs: [bool; 8],
    pub digital_input: [DigitalInput; 8],
    pub pulse_count: u8,
    pub last_measurement: [bool; 8],
}

impl Machine for Wago750_430UfmMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl Wago750_430UfmMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_750_430_UFM,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            inputs: self.inputs,
        }
    }

    pub fn emit_state(&mut self) {
        for (i, di) in self.digital_input.iter().enumerate() {
            self.inputs[i] = match di.get_value() {
                Ok(v) => v,
                Err(_) => false,
            };
        }

        let event = StateEvent {
            inputs: self.inputs,
        }
        .build();
        self.namespace.emit(Wago750_430UfmEvents::State(event));
    }
}
