use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_output::DigitalOutput;
use smol::channel::{Receiver, Sender};

use self::api::{StateEvent, Wago750_531MachineEvents, Wago750_531MachineNamespace};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_750_531_MACHINE,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_531Machine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: Wago750_531MachineNamespace,
    pub last_state_emit: Instant,
    pub outputs_on: [bool; 4],
    pub douts: [DigitalOutput; 4],
}

impl Machine for Wago750_531Machine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

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
            self.douts[index].set(on);
            self.emit_state();
        }
    }

    pub fn set_all_outputs(&mut self, on: bool) {
        self.outputs_on = [on; 4];
        for dout in &self.douts {
            dout.set(on);
        }
        self.emit_state();
    }
}
