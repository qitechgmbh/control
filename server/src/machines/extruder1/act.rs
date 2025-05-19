use super::{ExtruderV2, ExtruderV2Mode};
use control_core::{actors::Actor, converters::motor_converter::MotorConverter};
use std::time::{Duration, Instant};

// TODO: CLEAN UP ACT
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
            self.temp_sensor_4.act(now_ts).await;

            // self.heating_relay_1.act(now_ts).await;
            // self.heating_relay_2.act(now_ts).await;
            // self.heating_relay_3.act(now_ts).await;
            // self.heating_relay_4.act(now_ts).await;

            self.set_bar();

            self.heating_front.temperature = self.temp_sensor_1.temperature; // set the temperature read from the sensor            
            self.heating_middle.temperature = self.temp_sensor_2.temperature;
            self.heating_back.temperature = self.temp_sensor_3.temperature;
            self.heating_nozzle.temperature = self.temp_sensor_4.temperature;

            self.heating_front.temperature = 81.0; // set the temperature read from the sensor            
            self.heating_middle.temperature = 81.0;
            self.heating_back.temperature = 81.0;
            self.heating_nozzle.temperature = 81.0;

            self.temperature_controller_front.target_temp =
                self.heating_front.target_temperature as f64; // set target temperature

            self.temperature_controller_middle.target_temp =
                self.heating_middle.target_temperature as f64; // set target temperature

            self.temperature_controller_back.target_temp =
                self.heating_back.target_temperature as f64; // set target temperature

            self.temperature_controller_nozzle.target_temp =
                self.heating_nozzle.target_temperature as f64;

            self.set_can_switch_extrude();

            if self.mode == ExtruderV2Mode::Standby {
                self.turn_heating_off();
            } else if self.mode == ExtruderV2Mode::Heat || self.mode == ExtruderV2Mode::Extrude {
                let on_1 = self
                    .temperature_controller_front
                    .update(self.heating_front.temperature as f64, now_ts); // check if we need to set our relais to enabled to reach target temp

                let on_2 = self
                    .temperature_controller_middle
                    .update(self.heating_middle.temperature as f64, now_ts); // check if we need to set our relais to enabled to reach target temp

                let on_3 = self
                    .temperature_controller_back
                    .update(self.heating_back.temperature as f64, now_ts); // check if we need to set our relais to enabled to reach target temp

                let on_4 = self
                    .temperature_controller_nozzle
                    .update(self.heating_nozzle.temperature as f64, now_ts); // check if we need to set our relais to enabled to reach target temp

                // self.heating_relay_1.set(on_1); // set relay to on or off
                // self.heating_relay_2.set(on_2); // set relay to on or off
                // self.heating_relay_3.set(on_3); // set relay to on or off
                // self.heating_relay_4.set(on_4); // set relay to on or off

                self.heating_front.heating = on_1;
                self.heating_middle.heating = on_2;
                self.heating_back.heating = on_3;
                self.heating_nozzle.heating = on_4;
            }

            if self.mode == ExtruderV2Mode::Extrude && self.can_extrude == false {
                self.switch_to_heat();
            }

            let now = Instant::now();
            if now.duration_since(self.last_measurement_emit) > Duration::from_millis(32) {
                // channel 1
                self.emit_heating(self.heating_back.clone(), super::HeatingType::Back);
                self.emit_heating(self.heating_front.clone(), super::HeatingType::Front);
                self.emit_heating(self.heating_middle.clone(), super::HeatingType::Middle);
                self.emit_heating(self.heating_nozzle.clone(), super::HeatingType::Nozzle);

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

                    println!(
                        "pressuresensor bar: {}  new hz for motor {} target {}",
                        self.bar, res, self.pressure_motor_controller.target_pressure
                    );
                }

                self.last_measurement_emit = now;
            }
        })
    }
}
