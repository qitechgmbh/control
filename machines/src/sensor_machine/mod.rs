use std::time::Instant;

use ethercat_hal::io::analog_input::AnalogInput;
use smol::channel::{Receiver, Sender};

use crate::{
    AsyncThreadMessage, Machine, machine_identification::MachineIdentificationUnique,
};

pub mod act;
pub mod api;
pub mod new;

pub use new::SensorMachine;

impl Machine for SensorMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}