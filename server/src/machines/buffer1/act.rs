use std::time::{Duration, Instant};

use super::Buffer1;
use control_core::actors::Actor;
use tracing::info;

impl Actor for Buffer1 {
    fn act(
        &mut self,
        now: Instant
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            // TODO
            // if last measurement is older than 1 second, emit a new measurement
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0)
            {
                // info!("asdasd");
                // TODO
                self.last_measurement_emit = now;
            }
        })
    }
}
