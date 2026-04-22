use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::minimal_machines::test_machine_stepper::api::{AccelerationFactor, Frequency, Mode, ModeState, StateEvent, TestMachineStepperEvents};
use crate::{AsyncThreadMessage, Machine, MachineMessage};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::stepper_velocity_wago_750_671::StepperVelocityWago750671;
use ethercat_hal::io::stepper_velocity_wago_750_672::StepperVelocityWago750672;
use smol::channel::{Receiver, Sender};
use std::time::Instant;
pub mod act;
pub mod api;
pub mod new;
use crate::minimal_machines::test_machine_stepper::api::TestMachineStepperNamespace;
use crate::{TEST_MACHINE_STEPPER, VENDOR_QITECH};

#[derive(Debug)]
pub enum Stepper {
    Wago750_672(StepperVelocityWago750672),
    Wago750_671(StepperVelocityWago750671),
}

#[derive(Debug)]
pub struct TestMachineStepper {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: TestMachineStepperNamespace,
    pub last_state_emit: Instant,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub stepper: Stepper,
    pub mode: TestMachineMode,
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
        match &self.stepper {
            Stepper::Wago750_672(stepper_velocity_wago750672) => StateEvent {
                target_speed: stepper_velocity_wago750672.target_velocity,
                enabled: stepper_velocity_wago750672.enabled,
                freq: stepper_velocity_wago750672.freq_range_sel,
                acc_freq: stepper_velocity_wago750672.acc_range_sel,
                mode_state: ModeState {
                    mode: self.mode.clone().into(),
                },
            },
            Stepper::Wago750_671(stepper_velocity_wago750671) => StateEvent {
                target_speed: stepper_velocity_wago750671.target_velocity,
                enabled: stepper_velocity_wago750671.enabled,
                freq: stepper_velocity_wago750671.freq_range_sel,
                acc_freq: stepper_velocity_wago750671.acc_range_sel,
                mode_state: ModeState {
                    mode: self.mode.clone().into(),
            },
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();

        self.namespace.emit(TestMachineStepperEvents::State(event));
    }

    pub fn set_target_speed(&mut self, speed: i16) {
        match &mut self.stepper {
            Stepper::Wago750_672(stepper_velocity_wago750672) => {
                stepper_velocity_wago750672.set_velocity(speed)
            }
            Stepper::Wago750_671(stepper_velocity_wago750671) => {
                stepper_velocity_wago750671.set_velocity(speed)
            }
        };
        self.emit_state();
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode.clone();

        match mode {
            Mode::Standby => self.set_enabled(false),
            Mode::Hold => {
                self.set_enabled(true);
                self.stop_motor();
            }
            Mode::Turn => self.start_motor(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        match &mut self.stepper {
            Stepper::Wago750_672(stepper_velocity_wago750672) => {
                stepper_velocity_wago750672.set_enabled(enabled)
            }
            Stepper::Wago750_671(stepper_velocity_wago750671) => {
                stepper_velocity_wago750671.set_enabled(enabled)
            }
        };
        self.emit_state();
    }

    fn start_motor(&mut self) {
        self.stepper.start_motor();
        self.emit_state();
    }

    fn stop_motor(&mut self) {
        self.stepper.stop_motor();
        self.emit_state();
    }

    pub fn set_freq(&mut self, factor: Frequency) {
        match &mut self.stepper {
            Stepper::Wago750_672(stepper_velocity_wago750672) => {
                stepper_velocity_wago750672.set_freq_range_sel(factor)
            }
            Stepper::Wago750_671(stepper_velocity_wago750671) => {
                stepper_velocity_wago750671.set_freq_range_sel(factor)
            }
        };
        self.emit_state();
    }

    pub fn set_acc_freq(&mut self, factor: AccelerationFactor) {
        match &mut self.stepper {
            Stepper::Wago750_672(stepper_velocity_wago750672) => {
                stepper_velocity_wago750672.set_acc_range_sel(factor)
            }
            Stepper::Wago750_671(stepper_velocity_wago750671) => {
                stepper_velocity_wago750671.set_acc_range_sel(factor)
            }
        };
        self.emit_state();
    }
}
