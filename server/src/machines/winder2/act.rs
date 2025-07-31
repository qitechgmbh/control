use super::Winder2;
use control_core::{machines::new::MachineAct, uom_extensions::velocity::meter_per_minute};
use futures::executor::block_on;
use uom::si::f64::Velocity;
use std::time::{Duration, Instant};

impl MachineAct for Winder2 {
    fn act(&mut self, now: Instant) {
        // sync the spool speed
        self.sync_spool_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        // sync the traverse speed
        self.sync_traverse_speed();

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

            // send puller speed to buffer, if connected
            if let Some(connected) = &self.connected_buffer {
                if let Some(buffer_arc) = connected.machine.upgrade() {
                    let mut buffer = block_on(buffer_arc.lock());
                    buffer.buffer_lift_controller.set_target_output_speed(Velocity::get::<meter_per_minute>(&self.puller_speed_controller.get_target_speed()));
                }
            }
        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }
}
