use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::{
    digital_input::DigitalInput, digital_output::DigitalOutput,
    stepper_velocity_wago_750_671::StepperVelocityWago750671,
};
use smol::channel::{Receiver, Sender};

use self::api::{BottlecapsTestMachineEvents, BottlecapsTestMachineNamespace, StateEvent};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, TEST_MACHINE_BOTTLECAPS, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct BottlecapsTestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: BottlecapsTestMachineNamespace,
    pub last_state_emit: Instant,

    pub outputs: [bool; 8],
    pub inputs: [bool; 8],
    pub override_inputs: [bool; 8],
    pub douts: [DigitalOutput; 8],
    pub dins: [DigitalInput; 8],
    pub stepper: StepperVelocityWago750671,
}

impl Machine for BottlecapsTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl BottlecapsTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: TEST_MACHINE_BOTTLECAPS,
    };
}

impl BottlecapsTestMachine {
    /// Build the current state snapshot (used for serialization and events).
    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            inputs: self.inputs,
            override_inputs: self.override_inputs,
            stepper_enabled: self.stepper.enabled,
            stepper_target_speed: self.stepper.target_velocity,
            stepper_freq: self.stepper.freq_range_sel,
            stepper_acc_freq: self.stepper.acc_range_sel,
        }
    }

    /// Serialize and broadcast the current state to all subscribers.
    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(BottlecapsTestMachineEvents::State(event));
    }

    pub fn read_inputs(&mut self) {
        for (i, din) in self.dins.iter().enumerate() {
            self.inputs[i] = din.get_value().unwrap_or(false);
        }
    }

    pub fn set_override_input(&mut self, index: usize, on: bool) {
        if index < self.override_inputs.len() {
            self.override_inputs[index] = on;
        }
    }

    pub fn set_output(&mut self, index: usize, on: bool) {
        if index < self.outputs.len() {
            self.outputs[index] = on;
            self.douts[index].set(on);
            self.emit_state();
        }
    }

    pub fn stepper_set_target_speed(&mut self, speed: i16) {
        self.stepper.set_velocity(speed);
        self.emit_state();
    }

    pub fn stepper_set_enabled(&mut self, enabled: bool) {
        self.stepper.set_enabled(enabled);
        self.emit_state();
    }

    pub fn stepper_set_freq(&mut self, factor: u8) {
        self.stepper.set_freq_range_sel(factor);
        self.emit_state();
    }

    pub fn stepper_set_acc_freq(&mut self, factor: u8) {
        self.stepper.set_acc_range_sel(factor);
        self.emit_state();
    }
}
