use std::time::{Duration, Instant};

use super::BottlecapsTestMachine;
use crate::{MachineAct, MachineMessage, MachineValues};

impl MachineAct for BottlecapsTestMachine {
    fn act(&mut self, now: Instant) {
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        self.read_inputs();

        // Business logic: Enable output (Sort bottlecap out at position X) when it is detected at
        // the right position. External sorting logic (e.g. Image recognition) will trigger it via
        // the input.
        // TODO: Probably needs to be timed right in prod
        for (index, input) in self.inputs.clone().iter().enumerate() {
            let new_state = *input || self.override_inputs[index];
            if self.outputs[index] != new_state {
                self.set_output(index, new_state);
                self.emit_state();
                self.last_state_emit = now;
            }
        }

        // Emit state at ~30 Hz — change the divisor to adjust the rate.
        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 60.0) {
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
