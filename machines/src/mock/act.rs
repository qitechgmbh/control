use super::MockMachine;
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

/// Implements the `MachineAct` trait for the `MockMachine`.
///
/// # Parameters
/// - `_now_ts`: The current timestamp of type `Instant`.
///
/// # Returns
/// A pinned `Future` that resolves to `()` and is `Send`-safe. The future encapsulates the asynchronous behavior of the `act` method.
///
/// # Description
/// This method is called to perform periodic actions for the `MockMachine`. Specifically:
/// - It checks if the time elapsed since the last measurement emission exceeds 33 milliseconds.
/// - If the condition is met and the machine is in Running mode, it emits a sine wave data event.
/// - State events (frequency, mode) are only emitted when values change, not continuously.
///
/// The method ensures that the sine wave value is updated approximately 60 times per second (16ms intervals) when running.
///
impl MachineAct for MockMachine {
    fn act(&mut self, now: Instant) {
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };
        // Only emit live values if machine is in Running mode
        // The live values are updated approximately 30 times per second
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
