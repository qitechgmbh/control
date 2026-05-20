use std::time::{Duration, Instant};

use qitech_lib::machines::{Machine, MachineDataRegistry, MachineError};

use super::Wago750_501TestMachine;
use crate::MachineApi;

impl Machine for Wago750_501TestMachine {
    fn act(
        &mut self,
        _machine_data: Option<&mut MachineDataRegistry>,
    ) -> Result<(), MachineError> {
        let now = Instant::now();

        if let Ok(msg) = self.receiver.try_recv() {
            self.act_machine_message(msg);
        }

        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
        }

        Ok(())
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {}

    fn get_identification(&self) -> qitech_lib::machines::MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }
}
