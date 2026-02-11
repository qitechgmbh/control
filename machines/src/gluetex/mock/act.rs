use super::Gluetex;
use crate::{MachineAct, MachineMessage, MachineValues};
use std::time::{Duration, Instant};

impl MachineAct for Gluetex {
    fn act(&mut self, now: Instant) {
        let msg = self.api_receiver.try_recv();
        if let Ok(msg) = msg {
            self.act_machine_message(msg);
        }

        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => {
                self.namespace.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(machine_connection) => {
                if self.connected_machines.len() < self.max_connected_machines {
                    self.connected_machine_state.machine_identification_unique =
                        Some(machine_connection.ident.clone());
                    self.connected_machine_state.is_available = true;
                    self.connected_machines.push(machine_connection);
                }
                self.emit_state();
            }
            MachineMessage::DisconnectMachine(machine_connection) => {
                self.connected_machines
                    .retain(|connection| connection.ident != machine_connection.ident);
                if self.connected_machines.is_empty() {
                    self.connected_machine_state.machine_identification_unique = None;
                    self.connected_machine_state.is_available = false;
                }
                self.emit_state();
            }
            MachineMessage::RequestValues(sender) => {
                sender
                    .send_blocking(MachineValues {
                        state: serde_json::to_value(self.build_state_event())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.build_live_values_event())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
                sender.close();
            }
        }
    }
}
