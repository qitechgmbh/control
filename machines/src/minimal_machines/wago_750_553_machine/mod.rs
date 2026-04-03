use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::analog_output::AnalogOutput;
use smol::channel::{Receiver, Sender};

use self::api::{StateEvent, Wago750_553MachineEvents, Wago750_553MachineNamespace};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_750_553_MACHINE,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_553Machine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: Wago750_553MachineNamespace,
    pub last_state_emit: Instant,

    pub outputs: [f32; 4],
    pub aouts: [AnalogOutput; 4],
}

impl Machine for Wago750_553Machine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl Wago750_553Machine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_750_553_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            outputs: self.outputs,
            outputs_ma: self.outputs.map(|v| v * 20.0),
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(Wago750_553MachineEvents::State(event));
    }

    pub fn set_output(&mut self, index: usize, value: f32) {
        if index < self.outputs.len() {
            let clamped = value.clamp(0.0, 1.0);
            self.outputs[index] = clamped;
            self.aouts[index].set(clamped);
            self.emit_state();
        }
    }

    pub fn set_all_outputs(&mut self, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        self.outputs = [clamped; 4];
        for aout in &self.aouts {
            aout.set(clamped);
        }
        self.emit_state();
    }
}
