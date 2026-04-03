use super::{AquaPathV1, AquaPathV1Mode};
use crate::{MachineAct, MachineMessage, MachineValues};
use std::time::{Duration, Instant};

impl MachineAct for AquaPathV1 {
    fn act(&mut self, now_ts: Instant) {
        let msg = self.api_receiver.try_recv();
        if let Ok(msg) = msg {
            self.act_machine_message(msg);
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

        let front_notices = self.front_controller.drain_notices();
        let back_notices = self.back_controller.drain_notices();

        for notice in front_notices.iter().copied() {
            self.emit_controller_notice("Reservoir 2 (Front)", notice);
        }

        for notice in back_notices.iter().copied() {
            self.emit_controller_notice("Reservoir 1 (Back)", notice);
        }

        if !front_notices.is_empty() || !back_notices.is_empty() {
            self.emit_state();
        }

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
            MachineMessage::RequestValues(sender) => {
                sender
                    .send_blocking(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
                sender.close();
            }
        }
    }
}
