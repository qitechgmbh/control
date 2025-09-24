use crate::machines::extruder1::mock::ExtruderV2;
use control_core::machines::new::MachineAct;
use std::time::{Duration, Instant};

impl MachineAct for ExtruderV2 {
    fn act(&mut self, now: Instant) {
        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.maybe_emit_state_event();
            // Emit live values at 30 FPS
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }
}
