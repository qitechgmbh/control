use std::time::{Duration, Instant};

use super::BottlecapsTestMachine;
use crate::{MachineAct, MachineMessage, MachineValues};

impl MachineAct for BottlecapsTestMachine {
    /// Called every EtherCAT cycle. Drain messages and update at 30 Hz.
    fn act(&mut self, now: Instant) {
        // Drain the inbound message queue (API calls, subscriptions, etc.)
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        self.read_inputs();

        for (index, input) in self.inputs.clone().iter().enumerate() {
            self.set_output(index, *input);
        }

        // Emit state at ~30 Hz — change the divisor to adjust the rate.
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
                sender
                    .send_blocking(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::Value::Null,
                    })
                    .expect("Failed to send values");
                sender.close();
            }
        }
    }
}
