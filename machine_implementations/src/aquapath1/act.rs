use qitech_lib::machines::{Machine, MachineDataRegistry};
use super::{AquaPathV1, AquaPathV1Mode};
use crate::{ MachineApi};
use std::time::{Duration, Instant};

impl Machine for AquaPathV1 {
    fn act(&mut self, _reg : Option<&mut MachineDataRegistry> ) {
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
        self.front_controller.update(now);
        self.back_controller.update(now);

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

    fn react(&mut self, _registry: &qitech_lib::machines::MachineDataRegistry) {

    }

    fn get_identification(&self) -> qitech_lib::machines::MachineIdentificationUnique {
        self.machine_identification_unique
    }
}
