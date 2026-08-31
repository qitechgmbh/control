use super::AquaPathV1;
use qitech_framework::machine::{ActResult, Machine};
use std::time::{Duration, Instant};

impl Machine for AquaPathV1 {
    fn act(&mut self, _ : Duration) -> ActResult {
        let now = Instant::now();
        self.left_controller.update(now);
        self.right_controller.update(now);
        let left_notices = self.left_controller.drain_notices();
        let right_notices = self.right_controller.drain_notices();
        for notice in left_notices.iter().copied() {
            self.emit_controller_notice("Left Reservoir", notice);
        }
        for notice in right_notices.iter().copied() {
            self.emit_controller_notice("Right Reservoir", notice);
        }
        self.update_measurements();
        self.update_states(now);
        Ok(())
    }
}
