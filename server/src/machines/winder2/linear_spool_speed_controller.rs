use std::time::Instant;

use control_core::controllers::pid::PidController;
use uom::si::{
    angle::{degree, revolution},
    f64::Angle,
};

use super::{
    clamp_revolution::{clamp_revolution, scale_revolution_to_range, Clamping},
    spool_speed_controller::SpoolSpeedControllerTrait,
    tension_arm::TensionArm,
};

#[derive(Debug)]
pub struct LinearSpoolSpeedController {
    // current speed in steps/s
    speed: f64,
    min_speed: f64,
    max_speed: f64,
    enabled: bool,
    dampening_controller: PidController,
}

impl LinearSpoolSpeedController {
    pub fn new(min_speed: f64, max_speed: f64) -> Self {
        Self {
            min_speed,
            max_speed,
            speed: 0.0,
            enabled: false,
            dampening_controller: PidController::new((max_speed * 3e-8) as f64, 0.0, 0.0),
        }
    }
}

impl LinearSpoolSpeedController {
    fn speed_raw(&mut self, _t: Instant, tension_arm: &TensionArm) -> f64 {
        let min_speed = self.min_speed * 0.0;
        let max_speed = self.max_speed * 1.0;

        // calculate filament tension
        let tension_arm_min_degree: f64 = Angle::new::<degree>(20.0).get::<revolution>();
        let tension_arm_max_degree: f64 = Angle::new::<degree>(90.0).get::<revolution>();
        let tension_arm_angle = tension_arm.get_angle();
        let tension_arm_revolution = clamp_revolution(
            tension_arm_angle.get::<revolution>() as f32,
            tension_arm_min_degree as f32,
            tension_arm_max_degree as f32,
        );

        match tension_arm_revolution.1 {
            Clamping::Min => return min_speed,
            Clamping::Max => return min_speed,
            _ => {}
        };

        let filament_tension = scale_revolution_to_range(
            tension_arm_revolution.0 as f32,
            tension_arm_min_degree as f32,
            tension_arm_max_degree as f32,
        );

        let filament_tension_inverted = 1.0 - filament_tension;

        // interpolate speed
        let speed =
            filament_tension_inverted * (max_speed as f32 - min_speed as f32) + min_speed as f32;

        // save speed
        return speed as f64;
    }

    fn dampen_speed(&mut self, t: Instant, speed: f64) -> f64 {
        let error = speed - self.speed;
        let acceleration = self.dampening_controller.update(error as f64, t);
        let new_speed = self.speed + acceleration;
        return new_speed;
    }

    fn clamp_speed(&mut self, speed: f64) -> f64 {
        if speed < self.min_speed {
            return 0.0;
        } else if speed > self.max_speed {
            return self.max_speed;
        } else {
            return speed;
        }
    }
}

impl SpoolSpeedControllerTrait for LinearSpoolSpeedController {
    fn get_speed(&mut self, t: Instant, tension_arm: &TensionArm) -> i32 {
        let speed = self.speed_raw(t, tension_arm);
        let speed = match self.enabled {
            true => speed,
            false => 0.0,
        };
        let speed = self.dampen_speed(t, speed);

        // save speed before clamping or it will stay 0.0
        self.speed = speed;

        let speed = self.clamp_speed(speed);
        return speed as i32;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn reset(&mut self) {
        self.speed = 0.0;
        self.dampening_controller.reset();
    }

    fn set_max_speed(&mut self, max_speed: f32) {
        self.max_speed = max_speed as f64;
    }

    fn set_min_speed(&mut self, min_speed: f32) {
        self.min_speed = min_speed as f64;
    }

    fn get_max_speed(&self) -> f32 {
        self.max_speed as f32
    }

    fn get_min_speed(&self) -> f32 {
        self.min_speed as f32
    }
}
