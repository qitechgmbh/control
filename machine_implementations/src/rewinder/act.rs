use super::Rewinder;
use crate::MachineApi;
use qitech_lib::machines::{
    Machine, MachineDataRegistry, MachineError, MachineIdentificationUnique,
};
use std::time::Duration;

impl Machine for Rewinder {
    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique
    }

    fn act(&mut self, _machine_data: Option<&mut MachineDataRegistry>) -> Result<(), MachineError> {
        let now = std::time::Instant::now();
        if let Ok(machine_message) = self.api_receiver.try_recv() {
            self.act_machine_message(machine_message);
        }

        self.sync_puller_speed(now);
        self.sync_takeup_spool_speed(now);
        self.sync_source_spool_speed(now);
        self.sync_traverse_speed();
        self.stop_or_pull_rewind(now);
        self.log_rewind_diagnostics(now);

        if self.traverse_controller.did_change_state() {
            self.emit_state();
        }

        if self.can_rewind() != self.last_can_rewind {
            self.emit_state();
        }

        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
        Ok(())
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {}
}
