
use std::time::{Duration, Instant};
use crate::MachineAct;
use super::Winder2;

impl MachineAct for Winder2 {
    fn act(&mut self, now: Instant) {
        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: crate::MachineMessage) {
        todo!()
    }
}
