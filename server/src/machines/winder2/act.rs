use std::time::{Duration, Instant};

use super::Winder2;
use control_core::actors::Actor;
use tracing::debug;

impl Actor for Winder2 {
    fn act(
        &mut self,
        now: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.traverse.act(now).await;
            self.traverse_end_stop.act(now).await;
            self.puller.act(now).await;
            self.spool.act(now).await;
            self.tension_arm.analog_input_getter.act(now).await;
            self.laser.act(now).await;

            // sync the spool speed
            self.sync_spool_speed(now);

            // sync the puller speed
            self.sync_puller_speed(now);

            // sync the traverse speed
            self.sync_traverse_speed();

            // check if traverse state changed
            if self.traverse_controller.dif_change_state() {
                self.emit_traverse_state();

                // Also emit mode state when traverse state changes
                // This ensures the frontend gets updated capability flags when homing completes
                self.emit_mode_state();
            }

            // if last measurement emit is older than 1 second, emit a new measurement
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0)
            {
                self.emit_tension_arm_angle();
                self.emit_spool_rpm();
                self.emit_puller_speed();
                self.emit_traverse_position();
                self.last_measurement_emit = now;
            }
        })
    }
}
