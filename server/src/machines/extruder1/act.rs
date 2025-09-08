use control_core::machines::new::MachineAct;
use smol::future;
use std::pin::Pin;
use std::time::{Duration, Instant};

use crate::machines::extruder1::ExtruderV2;

impl MachineAct for ExtruderV2 {
    fn act(&mut self, now: Instant) -> Pin<&mut (dyn Future<Output = ()> + Send + '_)> {
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

        // refresh the slot so it's a "completed" Ready future again
        self.future_slot = future::ready(());

        // return pinned &mut reference as a trait object
        Pin::new(&mut self.future_slot) as Pin<&mut (dyn Future<Output = ()> + Send + '_)>
    }
}
