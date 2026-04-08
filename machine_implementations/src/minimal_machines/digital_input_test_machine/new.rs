use crate::{MachineHardware, MachineMessage, MachineNew};
use std::{cell::RefCell, rc::Rc};

use super::{DigitalInputTestMachine, api::DigitalInputTestMachineNamespace};
use qitech_lib::{
    ethercat_hal::devices::{ek1100::EK1100, el1008::EL1008, el2004::EL2004},
    machines::MachineIdentificationUnique,
};

impl MachineNew for DigitalInputTestMachine {
    fn new(hw: MachineHardware) -> Result<DigitalInputTestMachine, anyhow::Error> {
        let el1008: Rc<RefCell<EL1008>> = hw.try_get_ethercat_device_by_role(1)?;
        let el2004: Rc<RefCell<EL2004>> = hw.try_get_ethercat_device_by_role(2)?;
        let (tx, rx) = tokio::sync::mpsc::channel::<MachineMessage>(2);

        if hw.identification.machine_ident != DigitalInputTestMachine::MACHINE_IDENTIFICATION {
            return Err(anyhow::anyhow!("DigitalInputTestMachine: Passed Machine Hardware with ident: {:?} but expected: {:?}",hw.identification.machine_ident,DigitalInputTestMachine::MACHINE_IDENTIFICATION));
        }

        let my_test = Self {
            machine_identification_unique: hw.identification,
            led_on: [false; 4],
            digital_input_device: el1008,
            el2004,
            namespace: DigitalInputTestMachineNamespace { namespace: None },
            sender: tx,
            receiver: rx,
            last_state_emit: std::time::Instant::now(),
        };

        Ok(my_test)
    }
}
