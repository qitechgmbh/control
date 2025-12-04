use super::BufferV1;
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

impl MachineAct for BufferV1 {
    fn act(&mut self, now: Instant) {
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };
        // if last measurement is older than 1 second, emit a new measurement
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            // Emit live values at 30 FPS
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
            MachineMessage::ConnectToMachine(_machine_connection) =>
            /*Doesnt connect to any Machine so do nothing*/
            {
                ()
            }
            MachineMessage::DisconnectMachine(_machine_connection) =>
            /*Doesnt connect to any Machine so do nothing*/
            {
                ()
            }
        }
    }
}
