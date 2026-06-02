use std::time::Instant;

use units::length::meter;
use units::{Length, velocity::meter_per_second};

use crate::gluetex::Gluetex;

impl Gluetex {
    pub fn sync_puller_speed(&mut self, t: Instant) {
        let angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
        let steps_per_second = self
            .puller_speed_controller
            .converter
            .angular_velocity_to_steps(angular_velocity);
        let _ = self.puller.set_speed(steps_per_second);
    }

    pub fn sync_slave_puller_speed(&mut self, t: Instant) {
        let master_speed = self.puller_speed_controller.last_speed;

        let slave_velocity = self.slave_puller_speed_controller.update_speed(
            t,
            master_speed,
            &self.inlet_feeder_tension_arm,
        );

        let directed_velocity = if self.slave_puller_speed_controller.get_forward() {
            slave_velocity
        } else {
            -slave_velocity
        };

        let angular_velocity = self
            .slave_puller_speed_controller
            .velocity_to_angular_velocity(directed_velocity);

        let steps_per_second = self
            .slave_puller_speed_controller
            .converter
            .angular_velocity_to_steps(angular_velocity);

        if self.puller_speed_controller.is_enabled() {
            self.slave_puller.set_enabled(true);
            let _ = self.slave_puller.set_speed(steps_per_second);
        } else {
            self.slave_puller.set_enabled(false);
        }
    }

    pub fn sync_valve(&mut self, t: Instant) {
        let puller_speed = self.puller_speed_controller.last_speed;
        let dt = t.duration_since(self.valve_last_sync).as_secs_f64();
        let distance_moved =
            Length::new::<meter>(puller_speed.get::<meter_per_second>() * dt).abs();

        self.valve_controller
            .update_valve(&mut self.valve, distance_moved);

        self.valve_last_sync = t;
    }
}
