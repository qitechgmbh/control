use super::{ExtruderV2, ExtruderV2Mode};
use control_core::actors::Actor;
use std::time::{Duration, Instant};

impl Actor for ExtruderV2 {
    fn act(
        &mut self,
        now_ts: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.temperature_controller_back.update(now_ts).await;
            self.temperature_controller_nozzle.update(now_ts).await;
            self.temperature_controller_front.update(now_ts).await;
            self.temperature_controller_middle.update(now_ts).await;

            if self.mode == ExtruderV2Mode::Extrude {
                self.screw_speed_controller.update(now_ts, true).await;
            } else {
                self.screw_speed_controller.update(now_ts, false).await;
            }

            if self.mode == ExtruderV2Mode::Standby {
                self.turn_heating_off();
            }

            if self.mode == ExtruderV2Mode::Extrude
                && self.screw_speed_controller.get_motor_enabled() == false
            {
                self.switch_to_heat();
            }

            let now = Instant::now();
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0)
            {
                self.emit_inverter_status();
                self.emit_heating_element_power(super::HeatingType::Nozzle);
                self.emit_heating_element_power(super::HeatingType::Front);
                self.emit_heating_element_power(super::HeatingType::Middle);
                self.emit_heating_element_power(super::HeatingType::Back);

                self.emit_temperature(super::HeatingType::Nozzle);
                self.emit_temperature(super::HeatingType::Middle);
                self.emit_temperature(super::HeatingType::Back);
                self.emit_temperature(super::HeatingType::Front);

                self.emit_bar();
                self.emit_rpm();

                self.last_measurement_emit = now;
            }
        })
    }
}
