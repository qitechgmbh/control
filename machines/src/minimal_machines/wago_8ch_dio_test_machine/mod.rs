use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::{digital_input::DigitalInput, digital_output::DigitalOutput};
use smol::channel::{Receiver, Sender};

use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_8CH_IO_TEST_MACHINE,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
use self::api::{StateEvent, Wago8chDigitalIOTestMachineEvents, Wago8chDigitalIOTestMachineNamespace};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago8chDigitalIOTestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: Wago8chDigitalIOTestMachineNamespace,
    pub last_state_emit: Instant,
    pub digital_output: [DigitalOutput; 8],
    pub digital_input: [DigitalInput; 8],
}

impl Machine for Wago8chDigitalIOTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
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
                    self.digital_input[i]
                        .get_value()
                        .expect("digital input value should be available")
                })
                .collect::<Vec<_>>()
                .try_into()
                .expect("bool vector into array[8] should work"),
            digital_output: [
                self.digital_output[0].get(),
                self.digital_output[1].get(),
                self.digital_output[2].get(),
                self.digital_output[3].get(),
                self.digital_output[4].get(),
                self.digital_output[5].get(),
                self.digital_output[6].get(),
                self.digital_output[7].get(),
            ],
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(Wago8chDigitalIOTestMachineEvents::State(event));
    }

    pub fn set_output(&mut self, i: usize, value: bool) {
        self.digital_output[i].set(value);
    }
}
