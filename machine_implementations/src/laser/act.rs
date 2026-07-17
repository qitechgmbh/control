use std::time::{Duration, Instant};
use control_core::{MachineIdentificationUnique, machine::{Machine, MachineError}};
use super::LaserMachine;
use crate::MachineApi;

const REG_ERR_MESSAGE: &str = "Laser Couldnt write to the MachineDataRegistry";
impl Machine for LaserMachine {
    fn act(&mut self) -> Result<(), MachineError> {
        let now: Instant = std::time::Instant::now();
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };

        self.update();

        if self.did_change_state {
            self.emit_state();
        }

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }

        match &self.error {
            Some(e) => return Err(e.clone()),
            None => (),
        };

        Ok(())
    }

    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique
    }
}
