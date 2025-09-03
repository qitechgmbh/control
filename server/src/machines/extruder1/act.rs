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
                && !self.screw_speed_controller.get_motor_enabled()
            {
                self.switch_to_heat();
            }

            let now = Instant::now();

            // more than 33ms have passed since last emit (30 "fps" target)
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0)
            {
                self.maybe_emit_state_event();
                // Emit live values at 30 FPS
                self.emit_live_values();
                self.last_measurement_emit = now;
            }
        })
    }
}
