use std::time::Duration;
use std::time::Instant;

use qitech_framework::machine::ActResult;
use qitech_framework::machine::Machine;

use super::AquapathV1;

impl Machine for AquapathV1 {
    fn act(&mut self, _: Duration) -> ActResult {
        let now = Instant::now();

        for reservoir in self.reservoirs_mut() {
            reservoir.update(now);
        }
        self.emit_pending_notices();

        for reservoir in self.reservoirs_mut() {
            reservoir.publish(now);
        }
        self.mode_state.set(self.mode.clone());

        Ok(())
    }
}
