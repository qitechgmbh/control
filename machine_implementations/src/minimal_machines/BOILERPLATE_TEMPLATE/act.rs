// ============================================================================
// act.rs — The machine's update loop
// ============================================================================
// This file implements `MachineAct` which is called on every control cycle.
//
// `act()` is called at the EtherCAT cycle rate (typically 1 kHz). Keep it
// fast: drain the message queue, then emit state at the UI refresh rate.
//
// The `act_machine_message()` handler is mandatory boilerplate — only the
// `SubscribeNamespace` arm needs customization (emit your initial state there).
// ============================================================================

use std::time::{Duration, Instant};

use crate::{MachineAct, MachineMessage, MachineValues};
use super::MyMachine;

impl MachineAct for MyMachine {
    /// Called every EtherCAT cycle. Drain messages and update at 30 Hz.
    fn act(&mut self, now: Instant) {
        // Drain the inbound message queue (API calls, subscriptions, etc.)
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Emit state at ~30 Hz — change the divisor to adjust the rate.
        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            // TODO: read hardware values into your struct fields here if needed,
            // e.g.: self.read_inputs();
            self.emit_state();
            self.last_state_emit = now;
        }
    }

    /// Handle a single inbound message. This is mandatory plumbing — the only
    /// arm that typically needs editing is `SubscribeNamespace` (send initial
    /// state to a new subscriber) and `RequestValues` (serialize state for
    /// HTTP polling).
    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                // Send the current state immediately so the new subscriber
                // doesn't wait up to 33 ms for the next cycle.
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => {
                self.namespace.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection) => {
                // Cross-machine connections — not used in minimal machines.
            }
            MachineMessage::DisconnectMachine(_machine_connection) => {
                // Cross-machine connections — not used in minimal machines.
            }
            MachineMessage::RequestValues(sender) => {
                // Called by the HTTP polling path. Serialize current state.
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
