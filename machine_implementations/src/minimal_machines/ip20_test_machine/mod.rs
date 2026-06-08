use std::{cell::RefCell, rc::Rc, time::Instant};

use self::api::{IP20TestMachineEvents, IP20TestMachineNamespace, LiveValuesEvent, StateEvent};
use crate::{IP20_TEST_MACHINE, MachineMessage, QiTechMachine, VENDOR_QITECH};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::{
        devices::wago_modules::ip20_ec_di8_do8::IP20EcDi8Do8,
        io::{digital_input::DigitalInputDevice, digital_output::DigitalOutputDevice},
    },
    machines::{MachineIdentification, MachineIdentificationUnique},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct IP20TestMachine {
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: IP20TestMachineNamespace,
    pub last_state_emit: Instant,
    pub last_live_values_emit: Instant,
    pub outputs: [bool; 8],
    pub inputs: [bool; 8],
    pub device: Rc<RefCell<IP20EcDi8Do8>>,
}

impl QiTechMachine for IP20TestMachine {}

impl IP20TestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: IP20_TEST_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            outputs: self.outputs,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(IP20TestMachineEvents::State(event));
    }

    pub fn get_live_values(&self) -> LiveValuesEvent {
        LiveValuesEvent {
            inputs: self.inputs,
        }
    }

    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace
            .emit(IP20TestMachineEvents::LiveValues(event));
    }

    pub fn set_output(&mut self, index: usize, on: bool) {
        if index < self.outputs.len() {
            self.outputs[index] = on;
            self.device.borrow_mut().set_output(index, on);
            self.emit_state();
        }
    }

    pub fn set_all_outputs(&mut self, on: bool) {
        self.outputs = [on; 8];
        let mut dev = self.device.borrow_mut();
        for i in 0..8 {
            dev.set_output(i, on);
        }
        drop(dev);
        self.emit_state();
    }

    pub fn read_inputs(&mut self) {
        let dev = self.device.borrow();
        for i in 0..8 {
            self.inputs[i] = dev.get_input(i).unwrap_or(false);
        }
    }
}
