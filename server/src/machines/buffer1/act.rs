use std::time::{Duration, Instant};

use super::BufferV1;
use control_core::machines::new::MachineAct;
impl MachineAct for BufferV1 {
    fn act(&mut self, now: Instant) {
        // if last measurement is older than 1 second, emit a new measurement
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            // Emit live values at 30 FPS
            self.emit_live_values();

            self.last_measurement_emit = now;
        }
    }
}
