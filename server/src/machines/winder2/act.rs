use std::time::{Duration, Instant};

use super::Winder2;
use control_core::machines::new::MachineAct;

impl MachineAct for Winder2 {
    fn act(&mut self, now: Instant) {
        // sync the spool speed
        self.sync_spool_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        // sync the traverse speed
        self.sync_traverse_speed();

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

        if self.traverse_controller.did_change_state() {
            self.emit_state();
        }

        // sync the puller speed to the buffer output speed
        self.sync_buffer_speed();

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
            // check if traverse state changed
            if self.traverse_controller.did_change_state() {
                self.emit_state();
            }
        }
    }
}
