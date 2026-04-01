use std::time::Duration;
use qitech_lib::machines::Machine;
use qitech_lib::machines::MachineDataRegistry;
use super::ExtruderV3;

impl Machine for ExtruderV3 {
    fn act(&mut self, reg : Option<&mut MachineDataRegistry>) {
        let now = std::time::Instant::now();
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };

        self.temperature_controller_back.update(now);
        self.temperature_controller_nozzle.update(now);
        self.temperature_controller_front.update(now);
        self.temperature_controller_middle.update(now);

        if self.mode == super::ExtruderV3Mode::Extrude {
            self.screw_speed_controller.update(now, true);
        } else {
            self.screw_speed_controller.update(now, false);
        }

        if self.mode == super::ExtruderV3Mode::Standby {
            self.turn_heating_off();
        }

        if self.mode == super::ExtruderV3Mode::Extrude
            && !self.screw_speed_controller.get_motor_enabled()
        {
            self.switch_to_heat();
        }


        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.maybe_emit_state_event();
            // Emit live values at 30 FPS
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {
        // Extruder currently does not react to anything
    }

    fn get_identification(&self) -> qitech_lib::machines::MachineIdentificationUnique {
        self.identification
    }
}
