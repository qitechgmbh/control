use super::DryerMachine;
use crate::MachineApi;
use qitech_lib::machines::{Machine, MachineDataRegistry, MachineError, MachineIdentificationUnique};
use qitech_lib::modbus::ModbusDevice;
use std::time::{Duration, Instant};

impl Machine for DryerMachine {
    fn act(&mut self, _reg: Option<&mut MachineDataRegistry>) -> Result<(), MachineError> {
        let now = Instant::now();

        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        {
            let mut dryer = self.dryer.borrow_mut();
            let _ = dryer.handle_response();
            let _ = dryer.send_next_request();
        }

        if now.duration_since(self.last_emit) > Duration::from_secs(1) {
            self.update();
            self.emit_live_values();
            self.last_emit = now;
        }

        Ok(())
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {}

    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique
    }
}
