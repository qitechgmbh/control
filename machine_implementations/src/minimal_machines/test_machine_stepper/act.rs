use qitech_lib::machines::{
    Machine, MachineDataRegistry, MachineError, MachineIdentificationUnique,
};

use crate::MachineApi;

use super::TestMachineStepper;
use std::time::{Duration, Instant};

impl Machine for TestMachineStepper {
    fn act(&mut self, _machine: Option<&mut MachineDataRegistry>) -> Result<(), MachineError> {
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

    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn react(&mut self, _registry: &qitech_lib::machines::MachineDataRegistry) {}
}
