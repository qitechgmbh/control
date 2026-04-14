use qitech_lib::machines::{Machine, MachineDataRegistry, MachineIdentificationUnique};
use crate::MachineApi;
use super::LaserMachine;
use std::time::{Duration, Instant};



impl Machine for LaserMachine {
    fn act(&mut self, _reg: Option<&mut MachineDataRegistry>) {
        let now : Instant = std::time::Instant::now();
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };

        self.update();
        self.refresh_data();

        if self.did_change_state {
            self.emit_state();
        }

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {}

    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique
    }


}
