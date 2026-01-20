use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::{MachineAct, MachineMessage, MachineValues, wago_ai_test_machine::WagoAiTestMachine};

impl MachineAct for WagoAiTestMachine {
    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_measurement_rate();
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            crate::MachineMessage::ConnectToMachine(_machine_connection) => {}
            MachineMessage::DisconnectMachine(_machine_connection) => {}
            MachineMessage::RequestValues(sender) => {
                sender
                    .send_blocking(MachineValues {
                        state: serde_json::Value::Null,
                        live_values: serde_json::Value::Null,
                    })
                    .expect("Failed to send values");
                sender.close();
            }
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
            let mut values = [0.0; 4];
            let mut wiring_errors = [false; 4];

            // Read all 4 analog inputs
            for (i, ai) in self.analog_inputs.iter().enumerate() {
                let measured_value = ai.get_physical();
                wiring_errors[i] = ai.get_wiring_error();

                match measured_value {
                    ethercat_hal::io::analog_input::physical::AnalogInputValue::Potential(
                        _quantity,
                    ) => {
                        // Don't do anything - this module is current input only
                    }
                    ethercat_hal::io::analog_input::physical::AnalogInputValue::Current(
                        quantity,
                    ) => {
                        values[i] = quantity.value;
                    }
                }
            }

            let now_milliseconds = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Now is expected to be after UNIX_EPOCH")
                .as_millis();

            self.emit_analog_inputs(values, now_milliseconds);
            self.emit_wiring_errors(wiring_errors);
            self.last_measurement = Instant::now();
        }
    }
}
