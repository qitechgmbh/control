use super::MotorTestMachine;
use crate::{MachineAct, MachineMessage};
use std::time::Instant;

impl MachineAct for MotorTestMachine {
    fn act(&mut self, _now_ts: Instant) {
        // println!("[{}::act] Running act", module_path!());
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        self.motor_driver.set_enabled(self.motor_state.enabled);

        if self.motor_state.enabled {
            let steps_per_rev = 200.0; // Anpassen an Motor
            let steps_per_second = (self.motor_state.target_velocity as f64) * steps_per_rev / 60.0;

            let _ = self.motor_driver.set_speed(steps_per_second);
        } else {
            let _ = self.motor_driver.set_speed(0.0);
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(ns) => {
                self.namespace.namespace = Some(ns);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => {
                self.namespace.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _ = self.api_mutate(value);
            }
            _ => {}
        }
    }
}
