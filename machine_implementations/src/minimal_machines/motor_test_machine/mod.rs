use std::cell::RefCell;
use std::rc::Rc;

use crate::machine_identification::{MachineIdentification};
use crate::{ MachineMessage};
use crate::{MOTOR_TEST_MACHINE, VENDOR_QITECH};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::machines::MachineIdentificationUnique;
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug, Clone, PartialEq)]
pub struct MotorState {
    pub enabled: bool,
    pub target_velocity: i32,
}

#[derive(Debug)]
pub struct MotorTestMachine {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: api::BeckhoffNamespace,

    pub motor_driver: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    pub motor_driver_port: usize,
    pub motor_state: MotorState,
}


impl MotorTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MOTOR_TEST_MACHINE,
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
