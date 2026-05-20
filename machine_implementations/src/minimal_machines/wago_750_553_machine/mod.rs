use std::time::Instant;

use self::api::{StateEvent, Wago750_553MachineEvents, Wago750_553MachineNamespace};
use crate::{MachineMessage, QiTechMachine, VENDOR_QITECH, WAGO_750_553_MACHINE};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::{
        devices::wago_modules::wago_750_553::Wago750_553,
        io::analog_output::{AnalogOutputDevice, AnalogOutputOutput},
    },
    machines::{MachineIdentification, MachineIdentificationUnique},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_553Machine {
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: Wago750_553MachineNamespace,
    pub last_state_emit: Instant,
    pub outputs: [f32; 4],
    pub analog_output_device: Box<Wago750_553>,
}

impl QiTechMachine for Wago750_553Machine {}

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
            self.analog_output_device
                .set_output(index, AnalogOutputOutput(clamped));
            self.emit_state();
        }
    }

    pub fn set_all_outputs(&mut self, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        self.outputs = [clamped; 4];
        for i in 0..4 {
            self.analog_output_device
                .set_output(i, AnalogOutputOutput(clamped));
        }
        self.emit_state();
    }
}
