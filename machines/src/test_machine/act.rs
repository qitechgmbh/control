use super::TestMachine;
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

impl MachineAct for TestMachine {
    fn act(&mut self, now: Instant) {
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
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
            MachineMessage::ConnectToMachine(_machine_connection) => {
                // Does not connect to any Machine; do nothing
            }
            MachineMessage::DisconnectMachine(_machine_connection) => {
                // Does not connect to any Machine; do nothing
            }
        }
    }
}
