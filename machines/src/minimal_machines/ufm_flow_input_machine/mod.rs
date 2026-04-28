use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::ufm_flow_input::{UfmFlowData, UfmFlowInput};
use smol::channel::{Receiver, Sender};

use crate::{
    AsyncThreadMessage, Machine, MachineMessage, UFM_FLOW_INPUT_MACHINE, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
use self::api::{UfmFlowInputMachineEvents, UfmFlowInputMachineNamespace, StateEvent};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct UfmFlowInputMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: UfmFlowInputMachineNamespace,
    pub last_state_emit: Instant,

    pub flow_sensor: UfmFlowInput,
    pub last_data: UfmFlowData,
}

impl Machine for UfmFlowInputMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl UfmFlowInputMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: UFM_FLOW_INPUT_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            flow_lph: self.last_data.flow_lph,
            total_volume_m3: self.last_data.total_volume_m3,
            error: self.last_data.error,
            total_pulses: self.last_data.total_pulses,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(UfmFlowInputMachineEvents::State(event));
    }
}
