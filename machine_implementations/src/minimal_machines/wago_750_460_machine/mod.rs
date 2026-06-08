use std::{cell::RefCell, rc::Rc, time::Instant};

use self::api::{StateEvent, Wago750_460MachineEvents, Wago750_460MachineNamespace};
use crate::{MachineMessage, QiTechMachine, VENDOR_QITECH, WAGO_750_460_MACHINE};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::{
        devices::wago_modules::wago_750_460::Wago750_460,
        io::temperature_input::TemperatureInputDevice,
    },
    machines::{MachineIdentification, MachineIdentificationUnique},
};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Wago750_460Machine {
    // --- mandatory plumbing -------------------------------------------------
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: Wago750_460MachineNamespace,
    pub last_state_emit: Instant,

    // --- hardware -----------------------------------------------------------
    // Subdevices of a WAGO coupler are shared with the coupler
    pub temperature_input_device: Rc<RefCell<Wago750_460>>,
}

impl QiTechMachine for Wago750_460Machine {}

impl Wago750_460Machine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_750_460_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        let mut temperatures: [Option<f32>; 4] = [None; 4];
        let mut errors: [bool; 4] = [false; 4];
        for port in 0..4 {
            let input = self
                .temperature_input_device
                .borrow()
                .get_input(port)
                .expect("getting input for valid port should succeed");
            if input.error {
                errors[port] = true;
            } else {
                temperatures[port] = Some(input.temperature);
            }
        }
        StateEvent {
            temperatures,
            errors,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace.emit(Wago750_460MachineEvents::State(event));
    }
}
