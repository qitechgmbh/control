use crate::ip20_test_machine::api::{IP20TestMachineEvents, LiveValuesEvent, StateEvent};
use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::{AsyncThreadMessage, Machine, MachineMessage};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_input::DigitalInput;
use ethercat_hal::io::digital_output::DigitalOutput;
use smol::channel::{Receiver, Sender};
use std::time::Instant;
pub mod act;
pub mod api;
pub mod new;
use crate::ip20_test_machine::api::IP20TestMachineNamespace;
use crate::{IP20_TEST_MACHINE, VENDOR_QITECH};

#[derive(Debug)]
pub struct IP20TestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: IP20TestMachineNamespace,
    pub last_state_emit: Instant,
    pub last_live_values_emit: Instant,
    pub outputs: [bool; 8],
    pub inputs: [bool; 8],
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub douts: [DigitalOutput; 8],
    pub dins: [DigitalInput; 8],
}

impl Machine for IP20TestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl IP20TestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: IP20_TEST_MACHINE,
    };
}

impl IP20TestMachine {
    pub fn emit_state(&mut self) {
        let event = StateEvent {
            outputs: self.outputs,
        }
        .build();

        self.namespace.emit(IP20TestMachineEvents::State(event));
    }

    pub fn emit_live_values(&mut self) {
        let event = LiveValuesEvent {
            inputs: self.inputs,
        }
        .build();

        self.namespace
            .emit(IP20TestMachineEvents::LiveValues(event));
    }

    /// Set the state of a specific output
    pub fn set_output(&mut self, index: usize, on: bool) {
        if index < self.outputs.len() {
            self.outputs[index] = on;
            self.douts[index].set(on);
            self.emit_state();
        }
    }

    /// Set all outputs at once
    pub fn set_all_outputs(&mut self, on: bool) {
        self.outputs = [on; 8];
        for (dout, &value) in self.douts.iter().zip(self.outputs.iter()) {
            dout.set(value);
        }
        self.emit_state();
    }

    /// Read all digital inputs
    pub fn read_inputs(&mut self) {
        for (i, din) in self.dins.iter().enumerate() {
            self.inputs[i] = din.get_value().unwrap_or(false);
        }
    }
}
