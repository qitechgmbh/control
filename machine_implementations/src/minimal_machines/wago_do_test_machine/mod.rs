use std::time::Instant;

use self::api::{StateEvent, WagoDOTestMachineEvents, WagoDOTestMachineNamespace};
use crate::{MachineMessage, QiTechMachine, VENDOR_QITECH, WAGO_DO_TEST_MACHINE};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::{
        devices::wago_modules::wago_750_530::Wago750_530,
        io::digital_output::DigitalOutputDevice,
    },
    machines::{MachineIdentification, MachineIdentificationUnique},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct WagoDOTestMachine {
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: WagoDOTestMachineNamespace,
    pub last_state_emit: Instant,
    pub led_on: [bool; 8],
    pub digital_output_device: Box<Wago750_530>,
}

impl QiTechMachine for WagoDOTestMachine {}

impl WagoDOTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_DO_TEST_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            led_on: self.led_on,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(WagoDOTestMachineEvents::State(event));
    }

    pub fn set_led(&mut self, index: usize, on: bool) {
        if index < self.led_on.len() {
            self.led_on[index] = on;
            self.digital_output_device.set_output(index, on);
            self.emit_state();
        }
    }

    pub fn set_all_leds(&mut self, on: bool) {
        self.led_on = [on; 8];
        for i in 0..8 {
            self.digital_output_device.set_output(i, on);
        }
        self.emit_state();
    }
}
