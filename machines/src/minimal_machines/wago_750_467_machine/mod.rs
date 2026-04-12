use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::{
    devices::wago_modules::wago_750_467::Wago750_467,
    io::analog_input::{AnalogInput, physical::AnalogInputValue},
};
use smol::{
    channel::{Receiver, Sender},
    lock::RwLock,
};
use std::sync::Arc;

use self::api::{StateEvent, Wago750_467MachineEvents, Wago750_467MachineNamespace};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_750_467_MACHINE,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_467Machine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: Wago750_467MachineNamespace,
    pub last_state_emit: Instant,
    pub analog_inputs: [AnalogInput; 2],
    pub device: Arc<RwLock<Wago750_467>>,
}

impl Machine for Wago750_467Machine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl Wago750_467Machine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_750_467_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        let mut voltages = [0.0; 2];
        let mut normalized = [0.0; 2];
        let mut wiring_errors = [false; 2];

        for (index, analog_input) in self.analog_inputs.iter().enumerate() {
            normalized[index] = analog_input.get_normalized();
            wiring_errors[index] = analog_input.get_wiring_error();
            if let AnalogInputValue::Potential(value) = analog_input.get_physical() {
                voltages[index] = value.value;
            }
        }

        let raw_words = smol::block_on(async {
            let device = self.device.read().await;
            [device.get_raw_ai1(), device.get_raw_ai2()]
        });

        StateEvent {
            voltages,
            normalized,
            raw_words,
            wiring_errors,
        }
    }

    pub fn emit_state(&mut self) {
        self.namespace
            .emit(Wago750_467MachineEvents::State(self.get_state().build()));
    }
}
