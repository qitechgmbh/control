use super::Gluetex;
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

impl MachineAct for Gluetex {
    fn act(&mut self, now: Instant) {
        let machine_message = self.api_receiver.try_recv();
        match machine_message {
            Ok(machine_message) => self.act_machine_message(machine_message),
            Err(_e) => (),
        };
        // sync the spool speed
        self.sync_spool_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        // sync the slave puller speed
        self.sync_slave_puller_speed(now);

        // sync addon motor speeds
        self.sync_addon_motor_3_speed(now);
        self.sync_addon_motor_4_speed(now);

        // sync the traverse speed
        self.sync_traverse_speed();

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

        // update all temperature controllers
        self.temperature_controller_1.update(now);
        self.temperature_controller_2.update(now);
        self.temperature_controller_3.update(now);
        self.temperature_controller_4.update(now);
        self.temperature_controller_5.update(now);
        self.temperature_controller_6.update(now);

        if self.traverse_controller.did_change_state() {
            self.emit_state();
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
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(machine_connection) => {
                if self.connected_machines.len() >= self.max_connected_machines {
                    tracing::debug!(
                        "Refusing to add Machine Connection {:?}, since self.connected_machines would be over the limit of {:?}",
                        machine_connection,
                        self.max_connected_machines
                    );
                    return;
                }
                self.connected_machines.push(machine_connection);
            }
            MachineMessage::DisconnectMachine(_machine_connection) => {
                self.connected_machines.clear();
            }
        }
    }
}
