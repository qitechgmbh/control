use super::ExtruderV2;
use control_core::actors::Actor;
use std::time::Instant;

impl Actor for ExtruderV2 {
    fn act(
        &mut self,
        now_ts: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.inverter.act(now_ts).await;

            let now = chrono::Utc::now();
            if (now - self.last_measurement_emit).num_milliseconds() > 16 {
                self.emit_heating(self.heating_back.clone(), super::HeatingType::Back);
                self.emit_heating(self.heating_front.clone(), super::HeatingType::Front);
                self.emit_heating(self.heating_middle.clone(), super::HeatingType::Middle);

                self.emit_regulation();

                self.emit_mode_state();

                self.emit_rotation_state();

                self.emit_bar();
                self.emit_rpm();

                self.last_measurement_emit = now;
            }
        })
    }
}
