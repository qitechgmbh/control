use std::time::Instant;

use control_core::socketio::{event::Event, namespace::NamespaceCacheingLogic};
use ethercat_hal::io::analog_input::AnalogInput;
use smol::channel::{Receiver, Sender};

use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_AI_TEST_MACHINE,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
use self::api::{AnalogInputsEvent, WagoAiTestMachineEvents, WagoAiTestMachineNamespace};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct WagoAiTestMachine {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    main_sender: Option<Sender<AsyncThreadMessage>>,
    namespace: WagoAiTestMachineNamespace,

    last_measurement: Instant,
    measurement_rate_hz: f64,

    analog_inputs: [AnalogInput; 4],
}

impl Machine for WagoAiTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

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
                event.clone(),
            )));
    }

    pub fn emit_wiring_errors(&mut self, errors: [bool; 4]) {
        let event = AnalogInputsEvent::WiringErrors(errors[0], errors[1], errors[2], errors[3]);
        self.namespace
            .emit(WagoAiTestMachineEvents::State(Event::new(
                "WiringErrors",
                event.clone(),
            )));
    }

    pub fn emit_measurement_rate(&mut self) {
        let event = AnalogInputsEvent::MeasurementRateHz(self.measurement_rate_hz);
        self.namespace
            .emit(WagoAiTestMachineEvents::State(Event::new(
                "MeasurementRateHz",
                event.clone(),
            )));
    }
}
