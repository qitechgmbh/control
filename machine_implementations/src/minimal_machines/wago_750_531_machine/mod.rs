use std::{cell::RefCell, rc::Rc, time::Instant};

use self::api::{StateEvent, Wago750_531MachineEvents, Wago750_531MachineNamespace};
use crate::{MachineMessage, QiTechMachine, VENDOR_QITECH, WAGO_750_531_MACHINE};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::{
        devices::wago_modules::wago_750_531::Wago750_531,
        io::digital_output::DigitalOutputDevice,
    },
    machines::{MachineIdentification, MachineIdentificationUnique},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_531Machine {
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: Wago750_531MachineNamespace,
    pub last_state_emit: Instant,
    pub outputs_on: [bool; 4],
    pub digital_output_device: Rc<RefCell<Wago750_531>>,
}

impl QiTechMachine for Wago750_531Machine {}

impl Wago750_531Machine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_750_531_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            outputs_on: self.outputs_on,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(Wago750_531MachineEvents::State(event));
    }

    pub fn set_output(&mut self, index: usize, on: bool) {
        if index < self.outputs_on.len() {
            self.outputs_on[index] = on;
            self.digital_output_device.borrow_mut().set_output(index, on);
            self.emit_state();
        }
    }

    pub fn set_all_outputs(&mut self, on: bool) {
        self.outputs_on = [on; 4];
        let mut dev = self.digital_output_device.borrow_mut();
        for i in 0..4 {
            dev.set_output(i, on);
        }
        drop(dev);
        self.emit_state();
    }
}
