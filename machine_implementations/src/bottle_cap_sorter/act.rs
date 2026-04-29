use std::time::Instant;

use qitech_lib::machines::{Machine, MachineDataRegistry};
use crate::MachineApi;

use super::Sorter1;

impl Machine for Sorter1 {
    fn act(&mut self, _reg : Option<&mut MachineDataRegistry> ) {
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };

        let now = Instant::now();
        self.sync_conveyor_belt_speed(now);

        let mut valve_state_changed = false;
        let mut air_valves = self.air_valve_outputs.borrow_mut();
        for (index, valve_controller) in self.valve_controllers.iter_mut().enumerate() {
            if valve_controller.update(now) {
                air_valves.set_output(index, valve_controller.is_active());
                valve_state_changed = true;
            }
        }
        drop(air_valves);
        
        if valve_state_changed {
            self.emit_state();
        }

        self.trigger_valves(now);

        if now.duration_since(self.last_measurement_emit).as_millis() >= 100 {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn react(&mut self, _registry: &qitech_lib::machines::MachineDataRegistry) {
    }

    fn get_identification(&self) -> qitech_lib::machines::MachineIdentificationUnique {
        return self.machine_identification_unique
    }
}
