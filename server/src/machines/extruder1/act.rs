use control_core::machines::new::MachineAct;

use super::{ExtruderV2, ExtruderV2Mode};
use std::time::{Duration, Instant};

impl MachineAct for ExtruderV2 {
    fn act(
        &mut self,
        now_ts: Instant,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.temperature_controller_back.update(now_ts);
            self.temperature_controller_nozzle.update(now_ts);
            self.temperature_controller_front.update(now_ts);
            self.temperature_controller_middle.update(now_ts);

            if self.mode == ExtruderV2Mode::Extrude {
                self.screw_speed_controller.update(now_ts, true);
            } else {
                self.screw_speed_controller.update(now_ts, false);
            }

            if self.mode == ExtruderV2Mode::Standby {
                self.turn_heating_off();
            }

            if self.mode == ExtruderV2Mode::Extrude
                && self.screw_speed_controller.get_motor_enabled() == false
            {
                self.switch_to_heat();
            }

            if self.screw_speed_controller.get_wiring_error() {
                self.emit_state();
            }

            let now = Instant::now();

            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0)
            {
                // Emit live values at 60 FPS
                self.emit_live_values();
                self.last_measurement_emit = now;
            }
        })
    }
}
