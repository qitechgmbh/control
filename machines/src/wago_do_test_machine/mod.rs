use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_output::DigitalOutput;
use smol::channel::{Receiver, Sender};

use crate::{
    AsyncThreadMessage, WAGO_DO_TEST_MACHINE, Machine, MachineMessage, VENDOR_QITECH,
    wago_do_test_machine::api::{
        WagoDOTestMachineEvents, WagoDOTestMachineNamespace, StateEvent,
    },
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct WagoDOTestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: WagoDOTestMachineNamespace,
    pub last_state_emit: Instant,
    pub led_on: [bool; 8],
    pub douts: [DigitalOutput; 8],
}

impl Machine for WagoDOTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

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

    /// Set the state of a specific digital output
    pub fn set_led(&mut self, index: usize, on: bool) {
        if index < self.led_on.len() {
            self.led_on[index] = on;
            self.emit_state();
        }
    }

    /// Set all digital outputs at once
    pub fn set_all_leds(&mut self, on: bool) {
        self.led_on = [on; 8];
        self.emit_state();
    }
}