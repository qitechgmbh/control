use std::time::{Duration, Instant};
use crate::MachineAct;
use super::Winder2;
use crate::MachineMessage;

impl MachineAct for Winder2 {
    fn act(&mut self, now: Instant) {
        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: crate::MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
                tracing::info!("extruder1 received subscribe");
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;

                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection) => (),
            MachineMessage::DisconnectMachine(_machine_connection) =>
            /*Doesnt connec to any Machine do nothing*/
            {
                ()
            }
        }

    }
}
