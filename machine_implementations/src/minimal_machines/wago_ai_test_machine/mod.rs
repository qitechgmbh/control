use std::time::Instant;

use control_core::socketio::{event::Event, namespace::NamespaceCacheingLogic};
use tokio::sync::mpsc::{Receiver, Sender};

use self::api::{AnalogInputsEvent, WagoAiTestMachineEvents, WagoAiTestMachineNamespace};
use crate::{MachineMessage, QiTechMachine, VENDOR_QITECH, WAGO_AI_TEST_MACHINE};
use qitech_lib::{
    ethercat_hal::devices::wago_modules::wago_750_455::Wago750_455,
    machines::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct WagoAiTestMachine {
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: WagoAiTestMachineNamespace,
    pub last_measurement: Instant,
    pub measurement_rate_hz: f64,
    pub analog_input_device: Box<Wago750_455>,
}

impl QiTechMachine for WagoAiTestMachine {}

impl WagoAiTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_AI_TEST_MACHINE,
    };

    pub fn emit_analog_inputs(&mut self, values: [f64; 4], unix_timestamp_ms: u128) {
        let event = AnalogInputsEvent::AnalogInputs(
            values[0],
            values[1],
            values[2],
            values[3],
            unix_timestamp_ms.to_string(),
        );
        self.namespace
            .emit(WagoAiTestMachineEvents::State(Event::new(
                "AnalogInputs",
                event,
            )));
    }

    pub fn emit_wiring_errors(&mut self, errors: [bool; 4]) {
        let event = AnalogInputsEvent::WiringErrors(errors[0], errors[1], errors[2], errors[3]);
        self.namespace
            .emit(WagoAiTestMachineEvents::State(Event::new(
                "WiringErrors",
                event,
            )));
    }

    pub fn emit_measurement_rate(&mut self) {
        let event = AnalogInputsEvent::MeasurementRateHz(self.measurement_rate_hz);
        self.namespace
            .emit(WagoAiTestMachineEvents::State(Event::new(
                "MeasurementRateHz",
                event,
            )));
    }
}
