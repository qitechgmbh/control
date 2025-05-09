use super::ExtruderV2;
use control_core::actors::Actor;
use std::time::{Duration, Instant};

impl Actor for ExtruderV2 {
    fn act(
        &mut self,
        now_ts: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.inverter.act(now_ts).await;
            self.pressure_sensor.act(now_ts).await;
            self.temp_sensor_1.act(now_ts).await;
            self.temp_sensor_2.act(now_ts).await;
            self.temp_sensor_3.act(now_ts).await;
            self.heating_relay_1.act(now_ts).await;

            let now = Instant::now();
            if now.duration_since(self.last_measurement_emit) > Duration::from_millis(16) {
                // channel 1
                self.heating_front.temperature = self.temp_sensor_1.temperature; // set the temperature read from the sensor
                // channel 2
                self.heating_middle.temperature = self.temp_sensor_2.temperature;
                // channel 3
                self.heating_back.temperature = self.temp_sensor_3.temperature;

                println!(
                    "temp {} target {}",
                    self.heating_front.temperature, self.heating_front.target_temperature
                );
                let on = self
                    .temperature_controller
                    .update(self.heating_front.temperature as f64, now_ts); // check if we need to set our relais to enabled to reach target temp

                self.temperature_controller.target_temp =
                    self.heating_front.target_temperature as f64; // set target temperature

                self.heating_relay_1.set(on); // set relay to on or off

                self.emit_heating(self.heating_back.clone(), super::HeatingType::Back);
                self.emit_heating(self.heating_front.clone(), super::HeatingType::Front);
                self.emit_heating(self.heating_middle.clone(), super::HeatingType::Middle);
                self.emit_regulation();
                self.emit_mode_state();
                self.emit_rotation_state();
                self.set_bar();
                self.emit_rpm();
                self.last_measurement_emit = now;
            }
        })
    }
}
