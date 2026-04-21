#[cfg(not(feature = "mock-machine"))]
use super::WagoWinder;
#[cfg(not(feature = "mock-machine"))]
use crate::{MachineAct, MachineMessage, MachineValues};
#[cfg(not(feature = "mock-machine"))]
use std::time::{Duration, Instant};

#[cfg(not(feature = "mock-machine"))]
impl MachineAct for WagoWinder {
    fn act(&mut self, now: Instant) {
        let machine_message = self.api_receiver.try_recv();
        match machine_message {
            Ok(machine_message) => self.act_machine_message(machine_message),
            Err(_e) => (),
        };

        let traverse_in_error_recovery = !self.traverse.get_s1_bit3_speed_mode_ack()
            && ((self.traverse.get_status_byte2() & 0x80) != 0
                || (self.traverse.get_control_byte2() & 0x80) != 0);
        if traverse_in_error_recovery {
            self.traverse_controller.force_not_homed();
        }

        let traverse_should_be_energized = traverse_in_error_recovery
            || (self.traverse_mode != super::TraverseMode::Standby
                && ((self.traverse_controller.is_homed()
                    && !self.traverse_controller.is_going_in()
                    && !self.traverse_controller.is_going_out()
                    && !self.traverse_controller.is_traversing())
                    || self.traverse_controller.is_going_home()
                    || self.traverse_controller.is_going_in()
                    || self.traverse_controller.is_going_out()
                    || self.traverse_controller.is_traversing()));

        if self.traverse.is_enabled() != traverse_should_be_energized {
            self.traverse.set_enabled(traverse_should_be_energized);
        }
        self.traverse_controller
            .set_enabled(traverse_should_be_energized);

        self.sync_spool_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        if !traverse_in_error_recovery && traverse_should_be_energized {
            self.sync_traverse_speed();
        }

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

        if self.traverse_controller.did_change_state() {
            self.emit_state();
        }

        let axis_status_signature = self.current_axis_status_signature();
        if self.last_axis_status_signature.as_ref() != Some(&axis_status_signature) {
            self.emit_state();
            self.last_axis_status_signature = Some(axis_status_signature);
        }

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }

        self.emit_debug_snapshot_if_changed();
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
            MachineMessage::RequestValues(sender) => {
                sender
                    .send_blocking(MachineValues {
                        state: serde_json::to_value(self.build_state_event())
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
