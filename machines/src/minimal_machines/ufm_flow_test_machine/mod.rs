use std::time::Instant;

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::ufm_flow_input::UfmFlowInput;
use smol::channel::{Receiver, Sender};

use self::api::{StateEvent, UfmFlowTestMachineEvents, UfmFlowTestMachineNamespace};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, UFM_FLOW_TEST_MACHINE, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct UfmFlowTestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: UfmFlowTestMachineNamespace,
    pub last_state_emit: Instant,
    pub flow_lph: f64,
    pub total_volume_m3: f64,
    pub sensor_error: bool,
    pub flow_input: UfmFlowInput,
}

impl Machine for UfmFlowTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl UfmFlowTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: UFM_FLOW_TEST_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            flow_lph: self.flow_lph,
            total_volume_m3: self.total_volume_m3,
            sensor_error: self.sensor_error,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(UfmFlowTestMachineEvents::State(event));
    }
}
