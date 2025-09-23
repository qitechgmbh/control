use std::time::{Duration, Instant};

use crate::machines::buffer1::BufferV1Mode;

use super::BufferV1;
use control_core::machines::new::MachineAct;
use tracing::info;
use uom::si::velocity::millimeter_per_second;
impl MachineAct for BufferV1 {
    fn act(&mut self, now: Instant) {
        // sync the lift speed
        self.sync_lift_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        // sync the puller speed of winder
        self.sync_winder_puller();

        // check if lift state changed
        if self.buffer_lift_controller.did_change_state() {
            self.emit_state();
        }

        // if last measurement is older than 1 second, emit a new measurement
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0) {
            // Emit live values at 60 FPS
            self.emit_live_values();

            self.last_measurement_emit = now;
        }
    }
}
