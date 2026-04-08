use qitech_lib::machines::{Machine, MachineIdentificationUnique};

use super::Winder2;
use crate::{MachineApi};
use std::time::{Duration};


impl Machine for Winder2 {
    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn act(&mut self, machine_data: Option<&mut qitech_lib::machines::MachineDataRegistry>) {
        let now = std::time::Instant::now();
        let machine_message = self.api_receiver.try_recv();
            match machine_message {
                Ok(machine_message) => self.act_machine_message(machine_message),
                Err(_e) => (),
            };
        // sync the spool speed
        self.sync_spool_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        // sync the traverse speed
        self.sync_traverse_speed();

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

        if self.traverse_controller.did_change_state() {
            self.emit_state();
        }

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn react(&mut self, registry: &qitech_lib::machines::MachineDataRegistry) {

    }
}