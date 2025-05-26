use super::{ExtruderV2, ExtruderV2Mode};
use control_core::{actors::Actor, converters::motor_converter::MotorConverter};
use std::time::{Duration, Instant};

impl Actor for ExtruderV2 {
    fn act(
        &mut self,
        now_ts: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.inverter.act(now_ts).await;
            self.pressure_sensor.act(now_ts).await;
            self.temperature_controller_back.update(now_ts).await;
            self.temperature_controller_nozzle.update(now_ts).await;
            self.temperature_controller_front.update(now_ts).await;
            self.temperature_controller_middle.update(now_ts).await;

            self.set_bar();

            if self.mode == ExtruderV2Mode::Standby {
                self.turn_heating_off();
            }

            let now = Instant::now();
            if now.duration_since(self.last_measurement_emit) > Duration::from_millis(16) {
                // channel 1
                self.emit_heating(
                    self.temperature_controller_back.heating.clone(),
                    super::HeatingType::Back,
                );
                self.emit_heating(
                    self.temperature_controller_front.heating.clone(),
                    super::HeatingType::Front,
                );
                self.emit_heating(
                    self.temperature_controller_middle.heating.clone(),
                    super::HeatingType::Middle,
                );
                self.emit_heating(
                    self.temperature_controller_nozzle.heating.clone(),
                    super::HeatingType::Nozzle,
                );
                self.emit_regulation();
                self.emit_mode_state();
                self.emit_rotation_state();
                self.emit_bar();
                self.emit_rpm();

                if self.mode == ExtruderV2Mode::Extrude && self.uses_rpm == false {
                    self.pressure_motor_controller.target_pressure = self.target_bar as f64;
                    let res = self
                        .pressure_motor_controller
                        .update(self.bar as f64, now_ts);
                    let rpm = MotorConverter::hz_to_rpm(res as f32);
                    self.inverter.set_running_rpm_target(rpm);
                }

                self.last_measurement_emit = now;
            }
        })
    }
}
