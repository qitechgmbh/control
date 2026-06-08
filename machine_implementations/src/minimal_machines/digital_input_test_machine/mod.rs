use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::{DIGITAL_INPUT_TEST_MACHINE, MachineMessage, QiTechMachine, VENDOR_QITECH};
use api::DigitalInputTestMachineNamespace;
use qitech_lib::ethercat_hal::devices::el2004::EL2004;
use qitech_lib::ethercat_hal::io::digital_input::DigitalInputDevice;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use tokio::sync::mpsc::{Receiver, Sender};
pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct DigitalInputTestMachine {
    pub machine_identification_unique: MachineIdentificationUnique,
    pub led_on: [bool; 4],
    pub namespace: DigitalInputTestMachineNamespace,
    sender: Sender<MachineMessage>,
    receiver: Receiver<MachineMessage>,
    digital_input_device: Rc<RefCell<dyn DigitalInputDevice>>,
    el2004: Rc<RefCell<EL2004>>,
    last_state_emit: Instant,
}

impl DigitalInputTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: DIGITAL_INPUT_TEST_MACHINE,
    };
}

impl QiTechMachine for DigitalInputTestMachine {}
