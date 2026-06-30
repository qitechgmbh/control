use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_output::DigitalOutput;
use smol::channel::{Receiver, Sender};

use self::api::{StateEvent, Wago750_501TestMachineEvents, Wago750_501TestMachineNamespace};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_750_501_TEST_MACHINE,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_501TestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: Wago750_501TestMachineNamespace,
    pub last_state_emit: Instant,
    pub outputs: [bool; 2],
    pub douts: [DigitalOutput; 2],
}

impl Machine for Wago750_501TestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl Wago750_501TestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_750_501_TEST_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            outputs: self.outputs,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(Wago750_501TestMachineEvents::State(event));
    }

    pub fn set_output(&mut self, index: usize, on: bool) {
        if index < self.outputs.len() {
            self.outputs[index] = on;
            self.emit_state();
        }
    }

    pub fn set_all_outputs(&mut self, on: bool) {
        self.outputs = [on; 2];
        self.emit_state();
    }
}
