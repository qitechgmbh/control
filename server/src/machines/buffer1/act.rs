use std::{
    pin::Pin,
    time::{Duration, Instant},
};

use super::BufferV1;
use control_core::machines::new::MachineAct;
use smol::future;
impl MachineAct for BufferV1 {
    fn act(&mut self, now: Instant) -> Pin<&mut (dyn Future<Output = ()> + Send + '_)> {
        // if last measurement is older than 1 second, emit a new measurement
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            // Emit live values at 30 FPS
            self.emit_live_values();

            self.last_measurement_emit = now;
        }

        // refresh the slot so it's a "completed" Ready future again
        self.future_slot = future::ready(());

        // return pinned &mut reference as a trait object
        Pin::new(&mut self.future_slot) as Pin<&mut (dyn Future<Output = ()> + Send + '_)>
    }
}
