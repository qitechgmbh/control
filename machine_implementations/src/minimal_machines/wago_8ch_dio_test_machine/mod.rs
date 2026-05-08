use std::{time::Instant};

use self::api::{
    StateEvent, Wago8chDigitalIOTestMachineEvents, Wago8chDigitalIOTestMachineNamespace,
};
use crate::{
    MachineMessage, VENDOR_QITECH, WAGO_8CH_IO_TEST_MACHINE,
    machine_identification::MachineIdentification,
};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::{
        devices::wago_modules::wago_750_1506::Wago750_1506,
        io::{digital_input::DigitalInputDevice, digital_output::DigitalOutputDevice},
    },
    machines::MachineIdentificationUnique,
};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago8chDigitalIOTestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: Wago8chDigitalIOTestMachineNamespace,
    pub last_state_emit: Instant,
    pub digital_input_output_device: Box<Wago750_1506>,
    pub last_output_state: [bool; 8],
}

impl Wago8chDigitalIOTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_8CH_IO_TEST_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            digital_input: (0..8)
                .map(|i| {
                    self.digital_input_output_device
                        .get_input(i)
                        .expect("digital input value should be available for indicies 0 to 7")
                })
                .collect::<Vec<bool>>()
                .try_into()
                .expect("bool vector into array[8] should work"),
            digital_output: self.last_output_state,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(Wago8chDigitalIOTestMachineEvents::State(event));
    }

    pub fn set_output(&mut self, i: usize, value: bool) {
        self.digital_input_output_device.set_output(i, value);
        self.last_output_state[i] = value;
    }
}
