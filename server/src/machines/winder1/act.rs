use super::WinderV1;
use control_core::actors::Actor;

impl Actor for WinderV1 {
    fn act(
        &mut self,
        now_ts: u64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.traverse_driver.act(now_ts).await;
            self.puller_driver.act(now_ts).await;
            self.winder_driver.act(now_ts).await;
            self.tension_arm.analog_input_getter.act(now_ts).await;
            self.laser_driver.act(now_ts).await;

            // if last measurement emit is older than 1 second, emit a new measurement
            let now = chrono::Utc::now();
            if (now - self.last_measurement_emit).num_milliseconds() > 500 {
                self.emit_measurement_tension_arm();
                self.last_measurement_emit = now;
            }
        })
    }
}
