use qitech_lib::machines::{Machine, MachineError, MachineIdentificationUnique};
use super::Winder2;
use crate::{MACHINE_LASER_V1, MachineApi, laser::LaserData};
use std::time::{Duration, Instant};

impl Machine for Winder2 {
    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn act(
        &mut self,
        _machine_data: Option<&mut qitech_lib::machines::MachineDataRegistry>,
    ) -> Result<(), MachineError> {
        let now = std::time::Instant::now();
        let machine_message = self.api_receiver.try_recv();
        match machine_message {
            Ok(machine_message) => self.act_machine_message(machine_message),
            Err(_e) => (),
        };
        // sync the spool speed
        self.sync_spool_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        // sync the traverse speed
        self.sync_traverse_speed();

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

        if self.traverse_controller.did_change_state() {
            self.emit_state();
        }

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
        Ok(())
    }

    fn react(&mut self, registry: &qitech_lib::machines::MachineDataRegistry) {
        let laser_data : Result<LaserData, &'static str> = match self.laser_ident {
            Some(ident) => {
                registry.load(&ident)
            },
            None => {
                return;
            },
        };

        match laser_data {
            Ok(laser_data) => {
                let current = laser_data.live_values.diameter;
                let target = laser_data.state.laser_state.target_diameter;
                let lower = laser_data.state.laser_state.lower_tolerance;
                let upper = laser_data.state.laser_state.higher_tolerance;
                let last_speed = self.puller_speed_controller.last_speed;
                self.puller_speed_controller
                    .adaptive
                    .update_with_measurement(
                        current,
                        target,
                        lower,
                        upper,
                        last_speed,
                        Instant::now(),
                );                    
            }
            Err(_e) => {
                self.laser_ident = None;
            },
        }
    }
}
