use super::FF01;
use crate::MachineAct;
use crate::MachineMessage;
use std::time::{Duration, Instant};

impl MachineAct for FF01 {
    fn act(&mut self, now: Instant) {
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };

        self.update();

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
            MachineMessage::UnsubscribeNamespace => match &mut self.namespace.namespace {
                Some(namespace) => {
                    tracing::info!("UnsubscribeNamespace");
                    namespace.socket_queue_tx.close();
                    namespace.sockets.clear();
                    namespace.events.clear();
                }
                None => {},
            },
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::RequestValues(sender) => {},
        }
    }
}
