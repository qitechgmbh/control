use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::{AsyncThreadMessage, Machine, MachineMessage};
use crate::{MACHINE_BECKHOFF_TEST, VENDOR_QITECH};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use smol::channel::{Receiver, Sender};
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug, Clone, PartialEq)]
pub struct MotorState {
    pub enabled: bool,
    pub target_velocity: i32,
}

#[derive(Debug)]
pub struct BeckhoffMachine {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    main_sender: Option<Sender<AsyncThreadMessage>>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: api::BeckhoffNamespace,

    pub motor_driver: StepperVelocityEL70x1,
    pub motor_state: MotorState,
}

impl Machine for BeckhoffMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl BeckhoffMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_BECKHOFF_TEST,
    };

    pub fn emit_state(&mut self) {
        let event = api::StateEvent {
            motor_enabled: self.motor_state.enabled,
            motor_velocity: self.motor_state.target_velocity,
        };
        self.namespace
            .emit(api::BeckhoffEvents::State(event.build()));
    }

    pub fn turn_motor_on(&mut self) {
        self.motor_state.enabled = true;
        self.emit_state();
    }

    pub fn turn_motor_off(&mut self) {
        self.motor_state.enabled = false;
        self.emit_state();
    }
}
