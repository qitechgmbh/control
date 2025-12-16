use std::time::Instant;

use control_core::socketio::{event::Event, namespace::NamespaceCacheingLogic};
use ethercat_hal::io::analog_input::AnalogInput;
use smol::channel::{Receiver, Sender};

use crate::{
    ANALOG_INPUT_TEST_MACHINE, AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH,
    analog_input_test_machine::api::{
        AnalogInputTestMachineEvents, AnalogInputTestMachineNamespace, MeasurementEvent,
    },
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct AnalogInputTestMachine {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    main_sender: Option<Sender<AsyncThreadMessage>>,
    namespace: AnalogInputTestMachineNamespace,

    last_measurement: Instant,
    measurement_rate_hz: f64,

    analog_input: AnalogInput,
}

impl Machine for AnalogInputTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
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
