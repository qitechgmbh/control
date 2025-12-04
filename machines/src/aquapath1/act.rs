use super::{AquaPathV1, AquaPathV1Mode};
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

impl MachineAct for AquaPathV1 {
    fn act(&mut self, now_ts: Instant) {
        let msg = self.api_receiver.try_recv();
        match msg {
            Ok(msg) => {
                let _res = self.act_machine_message(msg);
            }
            Err(_) => (),
        };

        match self.mode {
            AquaPathV1Mode::Standby => {
                self.switch_to_standby();
            }
            AquaPathV1Mode::Auto => {
                self.switch_to_auto();
            }
        }

        let now = Instant::now();

        self.front_controller.update(now_ts);
        self.back_controller.update(now_ts);

        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
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
            MachineMessage::ConnectToMachine(_machine_connection) =>
            /*Doesnt connect to any Machine so do nothing*/
            {
                ()
            }
            MachineMessage::DisconnectMachine(_machine_connection) =>
            /*Doesnt connect to any Machine so do nothing*/
            {
                ()
            }
        }
    }
}
