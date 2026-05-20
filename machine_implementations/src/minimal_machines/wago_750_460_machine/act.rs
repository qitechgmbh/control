use std::time::{Duration, Instant};

use qitech_lib::machines::{Machine, MachineDataRegistry};

use crate::MachineApi;

use super::Wago750_460Machine;

impl Machine for Wago750_460Machine {
    fn act(&mut self, _machine_data: Option<&mut MachineDataRegistry>) {
        let now = Instant::now();

        if let Ok(msg) = self.receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Emit state at ~30 Hz.
        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
        }
    }

    fn react(&mut self, _registry: &qitech_lib::machines::MachineDataRegistry) {}

    fn get_identification(&self) -> qitech_lib::machines::MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }
}
