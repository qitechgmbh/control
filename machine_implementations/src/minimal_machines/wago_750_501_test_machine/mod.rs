use std::time::Instant;

use self::api::{StateEvent, Wago750_501TestMachineEvents, Wago750_501TestMachineNamespace};
use crate::{MachineMessage, QiTechMachine, VENDOR_QITECH, WAGO_750_501_TEST_MACHINE};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::{
        devices::wago_modules::wago_750_501::Wago750_501,
        io::digital_output::DigitalOutputDevice,
    },
    machines::{MachineIdentification, MachineIdentificationUnique},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_501TestMachine {
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: Wago750_501TestMachineNamespace,
    pub last_state_emit: Instant,
    pub outputs: [bool; 2],
    pub digital_output_device: Box<Wago750_501>,
}

impl QiTechMachine for Wago750_501TestMachine {}

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
            self.digital_output_device.set_output(index, on);
            self.emit_state();
        }
    }

    pub fn set_all_outputs(&mut self, on: bool) {
        self.outputs = [on; 2];
        for i in 0..2 {
            self.digital_output_device.set_output(i, on);
        }
        self.emit_state();
    }
}
