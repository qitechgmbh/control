use super::{AquaPathV1, AquaPathV1Mode};
use control_core::machines::new::MachineAct;
use std::time::{Duration, Instant};

impl MachineAct for AquaPathV1 {
    fn act(
        &mut self,
        now_ts: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            // if self.mode == AquaPathV1Mode::Standby {
            //     self.turn_cooling_off();
            //     self.turn_heating_off();
            // } else {
            //     self.turn_cooling_on();
            //     self.turn_heating_on();
            // }
            self.flow_controller_front.update(now_ts);
            self.flow_controller_back.update(now_ts);

            self.temp_controller_back.update(now_ts);
            self.temp_controller_front.update(now_ts);

            match self.mode {
                AquaPathV1Mode::Standby => {
                    self.switch_to_standby();
                }
                // AquaPathV1Mode::Cool => {
                //     self.switch_to_cool();
                // }
                // AquaPathV1Mode::Heat => {
                //     self.switch_to_heat();
                // }
                AquaPathV1Mode::Auto => {
                    self.switch_to_auto();
                }
            }

            let now = Instant::now();

            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0)
            {
                self.emit_live_values();

                self.last_measurement_emit = now;
            }
        })
    }
}
