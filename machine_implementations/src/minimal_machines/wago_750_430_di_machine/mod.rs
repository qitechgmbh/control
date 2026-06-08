use std::{cell::RefCell, rc::Rc, time::Instant};

use self::api::{StateEvent, Wago750_430DiMachineEvents, Wago750_430DiMachineNamespace};
use crate::{MachineMessage, QiTechMachine, VENDOR_QITECH, WAGO_750_430_DI_MACHINE};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::{
        devices::wago_modules::wago_750_430::Wago750_430,
        io::digital_input::DigitalInputDevice,
    },
    machines::{MachineIdentification, MachineIdentificationUnique},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_430DiMachine {
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: Wago750_430DiMachineNamespace,
    pub last_state_emit: Instant,
    pub digital_input_device: Rc<RefCell<Wago750_430>>,
}

impl QiTechMachine for Wago750_430DiMachine {}

impl Wago750_430DiMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_750_430_DI_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        let mut inputs = [false; 8];
        let dev = self.digital_input_device.borrow();
        for i in 0..8 {
            inputs[i] = dev.get_input(i).unwrap_or(false);
        }
        drop(dev);
        StateEvent { inputs }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(Wago750_430DiMachineEvents::State(event));
    }
}
