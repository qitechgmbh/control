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

            // automatically stops or pulls after N Meters if enabled
            self.stop_or_pull_spool(now);

            // more than 33ms have passed since last emit (30 "fps" target)
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0)
            {
                self.maybe_emit_state_event();
                self.emit_live_values();
                self.last_measurement_emit = now;
            }
        })
    }
}
