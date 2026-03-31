use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::{DIGITAL_INPUT_TEST_MACHINE, MachineApi, MachineMessage, QiTechMachine, VENDOR_QITECH};
use api::{DigitalInputTestMachineNamespace, StateEvent};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::ethercat_hal::devices::el2004::EL2004;
use qitech_lib::ethercat_hal::io::digital_input::DigitalInputDevice;
use qitech_lib::machines::{
    Machine, MachineDataRegistry, MachineIdentification, MachineIdentificationUnique,
};
use tokio::sync::mpsc::{Receiver, Sender};
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

impl Machine for DigitalInputTestMachine {
    fn act(&mut self, _registry: Option<&mut MachineDataRegistry>) {
        let now = std::time::Instant::now();

        let res = self.receiver.try_recv();
        match res {
            Ok(msg) =>  self.act_machine_message(msg),
            Err(_) => (),
        };

        let mut el2004 = self.el2004.borrow_mut();
        el2004.rxpdo.channel2 =
            Some(qitech_lib::ethercat_hal::pdo::basic::BoolPdoObject { value: true });

        let digital_input_device = self.digital_input_device.borrow_mut();
        let port_count = digital_input_device.get_port_count();
        for i in 0..port_count {
            let value = match digital_input_device.get_input(i) {
                Ok(v) => v,
                Err(_e) => false,
            };

            if i < 4 {
                self.led_on[i] = value;
            }
        }
        
        if now.duration_since(self.last_state_emit) > std::time::Duration::from_secs_f64(1.0 / 30.0) {
            self.namespace.emit(api::DigitalInputTestMachineEvents::State( StateEvent{ led_on: self.led_on }.build() ));       
            self.last_state_emit = now; 
        }
        
    }

    fn react(&mut self, _registry: &qitech_lib::machines::MachineDataRegistry) {
        // react to specific machines data
    }

    fn get_identification(&self) -> qitech_lib::machines::MachineIdentificationUnique {
        return self.machine_identification_unique.into();
    }
}

impl QiTechMachine for DigitalInputTestMachine {}
