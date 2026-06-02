use std::time::Instant;

use ethercat_hal::io::analog_input::physical::AnalogInputValue;
use units::electric_potential::volt;
use units::length::meter;
use units::{Length, velocity::meter_per_second};

use crate::gluetex::Gluetex;

impl Gluetex {
    pub fn sync_stepper_3_speed(&mut self, t: Instant) {
        let puller_angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);

        let puller_speed = self.puller_speed_controller.last_speed;
        let dt = t.duration_since(self.stepper_3_last_sync).as_secs_f64();
        let distance_moved =
            Length::new::<meter>(puller_speed.get::<meter_per_second>() * dt).abs();

        let endstop_hit = match self.stepper_3_analog_input.get_physical() {
            AnalogInputValue::Potential(voltage) => voltage.get::<volt>() < 1.0,
            _ => false,
        };

        if !self.puller_speed_controller.is_enabled() {
            tracing::debug!(
                controller_enabled = self.stepper_3_controller.is_enabled(),
                "stepper 3 force-disabled: puller controller off"
            );
            self.stepper_3.set_enabled(false);
            let _ = self.stepper_3.set_speed(0.0);
            self.stepper_3_last_sync = t;
            return;
        }

        self.stepper_3_controller.sync_motor_speed(
            &mut self.stepper_3,
            puller_angular_velocity,
            Some(endstop_hit),
            distance_moved,
        );

        self.stepper_3_last_sync = t;
    }

    pub fn sync_stepper_4_speed(&mut self, t: Instant) {
        let puller_angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);

        if !self.puller_speed_controller.is_enabled() {
            tracing::debug!(
                controller_enabled = self.stepper_4_controller.is_enabled(),
                "stepper 4 force-disabled: puller controller off"
            );
            self.stepper_4.set_enabled(false);
            let _ = self.stepper_4.set_speed(0.0);
            return;
        }

        self.stepper_4_controller.sync_motor_speed(
            &mut self.stepper_4,
            puller_angular_velocity,
            None,
            Length::new::<meter>(0.0),
        );
    }

    pub fn sync_stepper_5_speed(&mut self, t: Instant) {
        let master_speed = self.puller_speed_controller.last_speed;

        if !self.puller_speed_controller.is_enabled() {
            tracing::debug!(
                controller_enabled = self.stepper_5_controller.is_enabled(),
                "stepper 5 force-disabled: puller controller off"
            );
            self.stepper_5.set_enabled(false);
            let _ = self.stepper_5.set_speed(0.0);
            return;
        }

        let adjusted_speed = self.stepper_5_tension_controller.update_speed(
            t,
            master_speed,
            &self.tape_feeder_tension_arm,
        );
        let puller_angular_velocity = self
            .puller_speed_controller
            .speed_to_angular_velocity(adjusted_speed);

        self.stepper_5_controller.sync_motor_speed(
            &mut self.stepper_5,
            puller_angular_velocity,
            None,
            Length::new::<meter>(0.0),
        );
    }
}
