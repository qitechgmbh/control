use std::time::Instant;

use control_core::socketio::{ namespace::NamespaceCacheingLogic};
use ethercat_hal::io::digital_input::DigitalInput;
use smol::channel::{Receiver, Sender};

use crate::{
    DIGITAL_INPUT_TEST_MACHINE, AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH,
    digital_input_test_machine::api::{
        DigitalInputTestMachineEvents, DigitalInputTestMachineNamespace, StateEvent,
    },
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct DigitalInputTestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: DigitalInputTestMachineNamespace,
    pub last_state_emit: Instant,
    pub led_on:[bool;4],
    pub digital_input: [DigitalInput;4],
}

impl Machine for DigitalInputTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl DigitalInputTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: DIGITAL_INPUT_TEST_MACHINE,
    };

    pub fn emit_state(&mut self) {
        let event = StateEvent {
            led_on: self.led_on,
        }
        .build();

        self.namespace.emit(DigitalInputTestMachineEvents::State(event));
    }
}
