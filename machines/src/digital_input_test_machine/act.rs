use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::{MachineAct, digital_input_test_machine::DigitalInputTestMachine};

impl MachineAct for DigitalInputTestMachine {
    fn act_machine_message(&mut self, msg: crate::MachineMessage) {
        match msg {
            crate::MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
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
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
        }
    }
}
