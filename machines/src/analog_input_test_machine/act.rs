use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::{MachineAct, analog_input_test_machine::AnalogInputTestMachine};

impl MachineAct for AnalogInputTestMachine {
    fn act_machine_message(&mut self, msg: crate::MachineMessage) {
        match msg {
            crate::MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_measurement_rate();
            }
            crate::MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            crate::MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            crate::MachineMessage::ConnectToMachine(_machine_connection) => {}
            crate::MachineMessage::DisconnectMachine(_machine_connection) => {}
        }
    }

    fn act(&mut self, now: std::time::Instant) {
        let recv = self.api_receiver.try_recv();
        if let Ok(msg) = recv {
            self.act_machine_message(msg);
        }
        if now.duration_since(self.last_measurement)
            > Duration::from_secs_f64(1.0 / self.measurement_rate_hz)
        {
            let measured_value = self.analog_input.get_physical();
            match measured_value {
                ethercat_hal::io::analog_input::physical::AnalogInputValue::Potential(
                    _quantity,
                ) => {
                    // Don't do anything
                }
                ethercat_hal::io::analog_input::physical::AnalogInputValue::Current(quantity) => {
                    let now_milliseconds = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Now is expected to be after UNIX_EPOCH")
                        .as_millis();
                    self.emit_measurement(quantity.value, now_milliseconds);
                }
            }
            self.last_measurement = Instant::now();
        }
    }
}
