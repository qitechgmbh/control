use crate::MachineMessage;
use std::{cell::RefCell, rc::Rc};

use super::{DigitalInputTestMachine, api::DigitalInputTestMachineNamespace};
use qitech_lib::{
    ethercat_hal::devices::{EthercatDevice, downcast_rc_refcell, el1008::EL1008, el2004::EL2004},
    machines::MachineIdentificationUnique,
};

impl DigitalInputTestMachine {
    pub fn new(
        hw: Vec<Rc<RefCell<dyn EthercatDevice>>>,
    ) -> Result<DigitalInputTestMachine, anyhow::Error> {
        let dev = hw.get(1).cloned();
        let dev_1 = hw.get(2).cloned();
        let el1008: Rc<RefCell<EL1008>> = downcast_rc_refcell(dev.unwrap())?;
        let el2004: Rc<RefCell<EL2004>> = downcast_rc_refcell(dev_1.unwrap())?;
        let (tx, rx) = tokio::sync::mpsc::channel::<MachineMessage>(1);
        
        let my_test = Self {
            machine_identification_unique: MachineIdentificationUnique {
                machine_ident: DigitalInputTestMachine::MACHINE_IDENTIFICATION,
                serial: 420,
            },
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
