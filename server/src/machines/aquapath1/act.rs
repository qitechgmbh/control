use super::{AquaPathV1, AquaPathV1Mode};
use control_core::machines::new::MachineAct;
use std::time::{Duration, Instant};

impl MachineAct for AquaPathV1 {
    fn act(&mut self, now_ts: Instant) {
        match self.mode {
            AquaPathV1Mode::Standby => {
                self.switch_to_standby();
            }
            AquaPathV1Mode::Auto => {
                self.switch_to_auto();
            }
        }

        let now = Instant::now();

        self.front_controller.update(now_ts);
        self.back_controller.update(now_ts);

        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }
}
