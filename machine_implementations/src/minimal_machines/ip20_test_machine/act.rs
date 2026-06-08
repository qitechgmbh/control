use std::time::{Duration, Instant};

use qitech_lib::machines::{Machine, MachineDataRegistry, MachineError};

use super::IP20TestMachine;
use crate::MachineApi;

impl Machine for IP20TestMachine {
    fn act(
        &mut self,
        _machine_data: Option<&mut MachineDataRegistry>,
    ) -> Result<(), MachineError> {
        let now = Instant::now();

        if let Ok(msg) = self.receiver.try_recv() {
            self.act_machine_message(msg);
        }

        self.read_inputs();

        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
        }

        if now.duration_since(self.last_live_values_emit) > Duration::from_secs_f64(1.0 / 10.0) {
            self.emit_live_values();
            self.last_live_values_emit = now;
        }

        Ok(())
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {}

    fn get_identification(&self) -> qitech_lib::machines::MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }
}
