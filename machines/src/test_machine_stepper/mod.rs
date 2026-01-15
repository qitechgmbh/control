use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::test_machine_stepper::api::{StateEvent, TestMachineStepperEvents};
use crate::{AsyncThreadMessage, Machine, MachineMessage};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::devices::wago_modules::wago_750_671::Wago750_671;
use smol::channel::{Receiver, Sender};
use smol::lock::RwLock;
use std::sync::Arc;
use std::time::Instant;
pub mod act;
pub mod api;
pub mod new;
use crate::test_machine_stepper::api::TestMachineStepperNamespace;
use crate::{TEST_MACHINE_STEPPER, VENDOR_QITECH};

#[derive(Debug)]
pub struct TestMachineStepper {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: TestMachineStepperNamespace,
    pub last_state_emit: Instant,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub stepper: Arc<RwLock<Wago750_671>>,

    pub last_move: Instant,
    speed_state: SpeedCtlState,
    start_pulsed: bool,
    error_quit_pulsed: bool,
   reset_quit_pulsed: bool,

}

/// Minimal internal state to manage edges (Start / Error_Quit / Reset_Quit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpeedCtlState {
    Init,
    WaitReady,
    SelectMode,
    StartSpeed,
    Running,
    ErrorAck,
}


impl Machine for TestMachineStepper {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}
impl TestMachineStepper {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: TEST_MACHINE_STEPPER,
    };
}

impl TestMachineStepper {
    pub fn emit_state(&mut self) {
        let event = StateEvent {}.build();

        self.namespace.emit(TestMachineStepperEvents::State(event));
    }
}
