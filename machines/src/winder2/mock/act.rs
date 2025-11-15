use crate::winder2::mock::Winder2;
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

impl MachineAct for Winder2 {
    fn act(&mut self, now: Instant) {
        // more than 33ms have passed since last emit (30 "fps" target)
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
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(machine_connection) => {
                if self.connected_machines.len() >= self.max_connected_machines {
                    tracing::debug!(
                        "Refusing to add Machine Connection {:?}, since self.connected_machines would be over the limit of {:?}",
                        machine_connection,
                        self.max_connected_machines
                    );
                    return;
                }
                self.connected_machines.push(machine_connection);
            }
            MachineMessage::DisconnectMachine(_machine_connection) => {
                self.connected_machines.clear();
            }
        }
    }
}
