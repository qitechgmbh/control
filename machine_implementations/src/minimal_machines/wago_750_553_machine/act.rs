use std::time::{Duration, Instant};

use super::Wago750_553Machine;
use crate::{MachineAct, MachineMessage, MachineValues};

impl MachineAct for Wago750_553Machine {
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
            MachineMessage::UnsubscribeNamespace => {
                self.namespace.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::RequestValues(sender) => {
                let state_json =
                    serde_json::to_value(self.get_state()).expect("Failed to serialize state");
                smol::spawn(async move {
                    let _ = sender
                        .send(MachineValues {
                            state: state_json,
                            live_values: serde_json::Value::Null,
                        })
                        .await;
                    sender.close();
                })
                .detach();
            }
        }
    }
}
