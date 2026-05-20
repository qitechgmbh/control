use std::{cell::RefCell, rc::Rc, time::Instant};

use control_core::socketio::{event::Event, namespace::NamespaceCacheingLogic};
use qitech_lib::{
    ethercat_hal::io::analog_input::AnalogInputDevice,
    machines::{MachineIdentification, MachineIdentificationUnique},
};
use tokio::sync::mpsc::{Receiver, Sender};

use self::api::{AnalogInputTestMachineEvents, AnalogInputTestMachineNamespace, MeasurementEvent};
use crate::{ANALOG_INPUT_TEST_MACHINE, MachineMessage, QiTechMachine, VENDOR_QITECH};

pub mod act;
pub mod api;
pub mod new;

pub struct AnalogInputTestMachine {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: AnalogInputTestMachineNamespace,

    last_measurement: Instant,
    measurement_rate_hz: f64,

    analog_input: Rc<RefCell<dyn AnalogInputDevice>>,
}

impl AnalogInputTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: ANALOG_INPUT_TEST_MACHINE,
    };

    pub fn emit_measurement(&mut self, value: f64, unix_timestamp_ms: u128) {
        let event = MeasurementEvent::Measurement(value, unix_timestamp_ms.to_string());
        self.namespace
            .emit(AnalogInputTestMachineEvents::State(Event::new(
                "Measurement",
                event.clone(),
            )));
    }

    pub fn emit_measurement_rate(&mut self) {
        let event = MeasurementEvent::MeasurementRateHz(self.measurement_rate_hz);
        self.namespace
            .emit(AnalogInputTestMachineEvents::State(Event::new(
                "MeasurementRateHz",
                event.clone(),
            )));
    }
}

impl QiTechMachine for AnalogInputTestMachine {}
