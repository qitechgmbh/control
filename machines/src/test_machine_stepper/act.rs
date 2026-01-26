use smol::block_on;

use super::TestMachineStepper;
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

impl MachineAct for TestMachineStepper {
    fn act(&mut self, now: Instant) {
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
        }

        block_on(async {
            self.stepper.tick();
            let mut stm = self.stepper.device.write().await;
            self.stepper.target_acceleration = 10000;
        })
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection) => {}
            MachineMessage::DisconnectMachine(_machine_connection) => {}
            MachineMessage::RequestValues(sender) => {},
        }
    }
}
