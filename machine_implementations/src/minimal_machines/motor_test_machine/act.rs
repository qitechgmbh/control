use qitech_lib::machines::{Machine, MachineDataRegistry, MachineIdentificationUnique};

use super::MotorTestMachine;
use crate::{MachineApi};

impl Machine for MotorTestMachine {
    fn act(&mut self, _registry: Option<&mut MachineDataRegistry>) {
        // println!("[{}::act] Running act", module_path!());
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        let mut motor_driver_ref = self.motor_driver.borrow_mut();

        motor_driver_ref.set_enabled(self.motor_driver_port, self.motor_state.enabled);

        if self.motor_state.enabled {
            let steps_per_rev = 200.0; // Adjust to match motor
            let steps_per_second = (self.motor_state.target_velocity as f64) * steps_per_rev / 60.0;

            let _ = motor_driver_ref.set_speed(self.motor_driver_port, steps_per_second);
        } else {
            let _ = motor_driver_ref.set_speed(self.motor_driver_port, 0.0);
        }
    }

    fn react(&mut self, _registry: &qitech_lib::machines::MachineDataRegistry) { }

    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }
}
