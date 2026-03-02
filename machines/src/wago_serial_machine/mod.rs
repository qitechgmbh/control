use api::{StateEvent, WagoSerialMachineEvents, WagoSerialMachineNamespace};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::serial_interface::SerialInterface;
use smol::channel::{Receiver, Sender};
use std::time::Instant;

use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct WagoSerialMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: WagoSerialMachineNamespace,
    pub last_state_emit: Instant,
    pub serial_device: SerialInterface,
    serial_init_is_complete: bool,
    pub current_message: Option<String>,
}

impl Machine for WagoSerialMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl WagoSerialMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: 0x67,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            current_message: self.current_message.clone(),
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(WagoSerialMachineEvents::State(event));
    }

    pub fn send_message(&mut self, msg: String) {
        let msg_bytes = msg.into_bytes();
        let _res = smol::block_on((self.serial_device.write_message)(msg_bytes));
    }
}
