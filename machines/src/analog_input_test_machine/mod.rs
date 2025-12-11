use std::time::Instant;

use ethercat_hal::io::analog_input::AnalogInput;
use smol::channel::{Receiver, Sender};

use crate::{
    ANALOG_INPUT_TEST_MACHINE, AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH,
    analog_input_test_machine::api::AnalogInputTestMachineNamespace,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct AnalogInputTestMachine {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    main_sender: Option<Sender<AsyncThreadMessage>>,
    namespace: AnalogInputTestMachineNamespace,

    last_measurement: Instant,
    measurement_rate_hz: f64,

    analog_input: AnalogInput,
}

impl Machine for AnalogInputTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl AnalogInputTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: ANALOG_INPUT_TEST_MACHINE,
    };
}
