use super::LaserMachine;
use crate::MachineAct;
use crate::MachineMessage;
use crate::MachineValues;
use std::time::{Duration, Instant};

/// Implements the `MachineAct` trait for the `LaserMachine`.
///
/// # Parameters
/// - `_now_ts`: The current timestamp of type `Instant`.
///
/// # Returns
/// A pinned `Future` that resolves to `()` and is `Send`-safe. The future encapsulates the asynchronous behavior of the `act` method.
///
/// # Description
/// This method is called to perform periodic actions for the `LaserMachine`. Specifically:
/// - It checks if the time elapsed since the last measurement emission exceeds 20 milliseconds.
/// - If the condition is met, it asynchronously emits live values at 60 FPS.
///
/// The method ensures that the diameter value is updated approximately 60 times per second.
///
impl MachineAct for LaserMachine {
    fn act(&mut self, now: Instant) {
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };
        self.update();

        if self.did_change_state {
            self.emit_state();
        }

        if now.duration_since(self.last_machine_send) > Duration::from_millis(1)
        {
            for machine in self.connected_machines.iter()
            {
                use crate::LiveValues;

                let values = LiveValues::Laser(self.get_live_values());

                if machine.connection.len() <= 5
                {
                    _ = machine.connection.try_send(MachineMessage::ReceiveLiveValues(values));
                }
            }

            self.last_machine_send = now;
        }

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
                None => todo!(),
            },
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(machine_connection) =>
            {   
                if self.connected_machines.len() >= Self::MAX_CONNECTIONS
                {
                    tracing::debug!("Not adding machine connection. Max capacity reached!");
                    return;
                }

                self.connected_machines.push(machine_connection);
            }
            MachineMessage::DisconnectMachine(machine_connection) =>
            {
                self.connected_machines
                    .retain(|machine| 
                        machine.ident != machine_connection.ident);
            }
            MachineMessage::RequestValues(sender) => {
                sender
                    .send_blocking(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
                sender.close();

                ()
            }
            MachineMessage::ReceiveLiveValues(_) => {},
        }
    }
}
