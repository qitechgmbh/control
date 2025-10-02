#[cfg(not(feature = "mock-machine"))]
use crate::machines::extruder1::ExtruderV2;
#[cfg(not(feature = "mock-machine"))]
use control_core::machines::new::MachineAct;
#[cfg(not(feature = "mock-machine"))]
use std::time::{Duration, Instant};

#[cfg(not(feature = "mock-machine"))]
impl MachineAct for ExtruderV2 {
    fn act(&mut self, now: Instant) {
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

        let now = Instant::now();

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.maybe_emit_state_event();
            // Emit live values at 30 FPS
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }
}
