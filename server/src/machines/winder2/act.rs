use std::time::{Duration, Instant};

use super::Winder2;
use control_core::machines::new::MachineAct;

impl MachineAct for Winder2 {
    fn act(
        &mut self,
        now: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            // sync the spool speed
            self.sync_spool_speed(now);

            // sync the puller speed
            self.sync_puller_speed(now);

            // sync the traverse speed
            self.sync_traverse_speed();

            // calculate the progress of meters
            self.auto_stop_or_pull(now);

            // check if traverse state changed
            if self.traverse_controller.did_change_state() {
                self.emit_state();
            }

            // if last measurement emit is older than 1 second, emit a new measurement
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0)
            {
                self.emit_live_values();
                self.last_measurement_emit = now;
            }
        })
    }
}
