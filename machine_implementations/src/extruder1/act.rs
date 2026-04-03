use qitech_lib::machines::{Machine, MachineDataRegistry, MachineIdentificationUnique};
use crate::{MachineApi, extruder1::ExtruderV2};
use std::time::{Duration, Instant};

impl Machine for ExtruderV2 {
    fn act(&mut self, registry : Option<&mut MachineDataRegistry>) {
        let now = Instant::now();
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };

        let mut relais = self.relais_output.borrow_mut();
        let mut temp_sensor = self.temperature_input.borrow();
        self.temperature_controller_back.update(now,&mut relais,&temp_sensor);
        self.temperature_controller_nozzle.update(now,&mut relais,&temp_sensor);
        self.temperature_controller_front.update(now,&mut relais,&temp_sensor);
        self.temperature_controller_middle.update(now,&mut relais,&temp_sensor);
        drop(relais);
        drop(temp_sensor);

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

        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.update_total_energy(now);
            self.maybe_emit_state_event();
            // Emit live values at 30 FPS
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn get_identification(&self) ->  MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn react(&mut self, registry: &MachineDataRegistry) {
        
    }
}
