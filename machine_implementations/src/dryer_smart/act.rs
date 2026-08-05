use super::DryerSmartMachine;
use crate::MachineApi;
use qitech_lib::machines::{
    Machine, MachineDataRegistry, MachineError, MachineIdentificationUnique,
};
use std::time::Instant;

impl Machine for DryerSmartMachine {
    fn act(&mut self, _reg: Option<&mut MachineDataRegistry>) -> Result<(), MachineError> {
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        self.core.poll_device()?;

        if self.core.tick_due(Instant::now()) {
            self.update();
            self.core.check_auto_stop();
            self.emit_live_values();
        }

        Ok(())
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {}

    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique
    }
}
