use std::time::{Duration, Instant};

use crate::machines::buffer1::BufferV1Mode;

use super::BufferV1;
use control_core::machines::new::MachineAct;
use tracing::info;
use uom::si::velocity::millimeter_per_second;
impl MachineAct for BufferV1 {
    fn act(&mut self, now: Instant) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {

            // sync the puller speed
            self.sync_puller_speed(now);

            if self.mode == BufferV1Mode::FillingBuffer {
                // get the current input speed of the buffer ( = current output speed of the puller)
                self.buffer_lift_controller.set_current_input_speed(self.puller_speed_controller.get_target_speed().get::<millimeter_per_second>());
                // calculate the speed of the buffer lift
                let speed = self.buffer_lift_controller.calculate_buffer_lift_speed();
                // Debug
                info!(
                    "Buffer Lift Speed: {:?}",
                    self.buffer_lift_controller.get_lift_speed().get::<millimeter_per_second>()
                );
                // update the speed of the buffer lift
                self.buffer_lift_controller.update_speed(speed);
            }
            // if last measurement is older than 1 second, emit a new measurement
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0)
            {
                // Emit live values at 60 FPS
                self.emit_live_values();

            self.last_measurement_emit = now;
        }
    }
}
