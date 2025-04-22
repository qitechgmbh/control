use std::time::Instant;

use super::Winder2;
use control_core::actors::Actor;

impl Actor for Winder2 {
    fn act(
        &mut self,
        now: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            // self.traverse_driver.act(now).await;
            // self.puller_driver.act(now).await;
            self.spool.act(now).await;
            self.tension_arm.analog_input_getter.act(now).await;
            self.laser.act(now).await;

            // sync the spool speed
            self.sync_spool_speed(now);

            // if last measurement emit is older than 1 second, emit a new measurement
            let now = chrono::Utc::now();
            if (now - self.last_measurement_emit).num_milliseconds() > 16 {
                self.emit_tension_arm_angle();
                self.emit_spool_rpm();
                self.last_measurement_emit = now;
            }
        })
    }
}
