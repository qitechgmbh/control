use super::Winder2;
use control_core::actors::Actor;

impl Actor for Winder2 {
    fn act(
        &mut self,
        now_ts: u64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            // self.traverse_driver.act(now_ts).await;
            // self.puller_driver.act(now_ts).await;
            self.spool.act(now_ts).await;
            self.tension_arm.analog_input_getter.act(now_ts).await;
            self.laser.act(now_ts).await;

            // sync the spool speed
            self.sync_spool_speed(now_ts);

            // if last measurement emit is older than 1 second, emit a new measurement
            let now = chrono::Utc::now();
            if (now - self.last_measurement_emit).num_milliseconds() > 16 {
                self.emit_tension_arm();
                self.emit_spool_rpm();
                self.last_measurement_emit = now;
            }
        })
    }
}
