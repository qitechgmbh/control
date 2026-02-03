use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::test_machine_stepper::api::{StateEvent, TestMachineStepperEvents};
use crate::{AsyncThreadMessage, Machine, MachineMessage};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::stepper_velocity_wago_750_672::StepperVelocityWago750672;
use smol::channel::{Receiver, Sender};
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
    pub stepper: StepperVelocityWago750672,
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
    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            target_speed: self.stepper.target_velocity,
            enabled: self.stepper.enabled,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();

        self.namespace.emit(TestMachineStepperEvents::State(event));
    }

    pub fn set_target_speed(&mut self, speed: i16) {
        self.stepper.set_velocity(speed);
        self.emit_state();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        tracing::error!("Enabling driver now.");
        self.stepper.set_enabled(enabled);
        self.emit_state();
    }
}
