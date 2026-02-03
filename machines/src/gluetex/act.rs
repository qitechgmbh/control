use super::Gluetex;
use crate::{MachineAct, MachineMessage, MachineValues};
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
        self.sync_addon_motor_5_speed(now);

        // sync the traverse speed
        self.sync_traverse_speed();

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

        // check tension arm positions and trigger emergency stop if needed
        self.check_tension_arm_monitor();

        // check sleep timer and enter standby if inactive for too long
        self.check_sleep_timer(now);

        // auto-reset sleep timer if there's activity
        if self.detect_activity() {
            self.reset_sleep_timer();
        }

        // update all temperature controllers
        self.temperature_controller_1.update(now);
        self.temperature_controller_2.update(now);
        self.temperature_controller_3.update(now);
        self.temperature_controller_4.update(now);
        self.temperature_controller_5.update(now);
        self.temperature_controller_6.update(now);

        // Check for completed auto-tuning and emit results
        self.check_autotuning_results();

        if self.traverse_controller.did_change_state() {
            self.emit_state();
            self.last_state_emit = now;
        }

        let autotuning_active = self.temperature_controller_1.is_autotuning()
            || self.temperature_controller_2.is_autotuning()
            || self.temperature_controller_3.is_autotuning()
            || self.temperature_controller_4.is_autotuning()
            || self.temperature_controller_5.is_autotuning()
            || self.temperature_controller_6.is_autotuning();

        // Emit state regularly when autotuning or when sleep timer is enabled
        let should_emit_state = autotuning_active || self.sleep_timer_config.enabled;

        if should_emit_state
            && now.duration_since(self.last_state_emit) > Duration::from_millis(500)
        {
            self.emit_state();
            self.last_state_emit = now;
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
            }
        }
    }
}
