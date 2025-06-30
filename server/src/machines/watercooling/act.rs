use super::{WaterCooling, WaterCoolingMode};
use control_core::actors::Actor;
use std::time::{Duration, Instant};

impl Actor for WaterCooling {
    fn act(
        &mut self,
        now_ts: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.temperature_controller.update(now_ts).await;

            if self.mode == WaterCoolingMode::Standby {
                self.turn_cooling_off();
            }

            let now = Instant::now();

            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0)
            {
                self.emit_cooling(self.temperature_controller.cooling.clone());

                self.emit_cooling_element_power();

                self.emit_mode_state();

                self.last_measurement_emit = now;
            }
        })
    }
}
