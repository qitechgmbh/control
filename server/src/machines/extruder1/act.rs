#[cfg(not(feature = "mock-machine"))]
use crate::machines::extruder1::ExtruderV2;
#[cfg(not(feature = "mock-machine"))]
use control_core::machines::new::MachineAct;
#[cfg(not(feature = "mock-machine"))]
use std::time::{Duration, Instant};

#[cfg(not(feature = "mock-machine"))]
impl MachineAct for ExtruderV2 {
    fn act(&mut self, now: Instant) {
        use std::sync::atomic::AtomicU64;
        static CYCLE_COUNTER: AtomicU64 = AtomicU64::new(0);
        let start = Instant::now();

        self.temperature_controller_back.update(now);
        self.temperature_controller_nozzle.update(now);
        self.temperature_controller_front.update(now);
        self.temperature_controller_middle.update(now);

        if self.mode == super::ExtruderV2Mode::Extrude {
            self.screw_speed_controller.update(now, true);
        } else {
            self.screw_speed_controller.update(now, false);
        }

        if self.mode == super::ExtruderV2Mode::Standby {
            self.turn_heating_off();
        }

        if self.mode == super::ExtruderV2Mode::Extrude
            && !self.screw_speed_controller.get_motor_enabled()
        {
            self.switch_to_heat();
        }

        /*// emit state & live values every ~33ms
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            // self.maybe_emit_state_event();
            self.emit_live_values();
            self.last_measurement_emit = now;
        }*/

        // End timer & log every 10 cycles
        let elapsed = start.elapsed();
        let count = CYCLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if count % 1000 == 0 {
            tracing::info!(
                "[ExtruderV2::act] Duration of act(): {} ns",
                elapsed.as_nanos()
            );
        }
    }
}
