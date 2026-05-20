use std::{cell::RefCell, rc::Rc, time::Instant};

use self::api::{StateEvent, TestMachineEvents, TestMachineNamespace};
use crate::{MachineMessage, QiTechMachine, TEST_MACHINE, VENDOR_QITECH};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::{devices::el2004::EL2004, io::digital_output::DigitalOutputDevice},
    machines::{MachineIdentification, MachineIdentificationUnique},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct TestMachine {
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: TestMachineNamespace,
    pub last_state_emit: Instant,
    pub led_on: [bool; 4],
    pub el2004: Rc<RefCell<EL2004>>,
}

impl QiTechMachine for TestMachine {}

impl TestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: TEST_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            led_on: self.led_on,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(TestMachineEvents::State(event));
    }

    pub fn set_led(&mut self, index: usize, on: bool) {
        if index < self.led_on.len() {
            self.led_on[index] = on;
            self.el2004.borrow_mut().set_output(index, on);
            self.emit_state();
        }
    }

    pub fn set_all_leds(&mut self, on: bool) {
        self.led_on = [on; 4];
        let mut dev = self.el2004.borrow_mut();
        for i in 0..4 {
            dev.set_output(i, on);
        }
        drop(dev);
        self.emit_state();
    }
}
