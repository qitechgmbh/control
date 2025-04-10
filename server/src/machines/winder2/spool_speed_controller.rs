use std::fmt::Debug;

use control_core::controllers::pid::PidController;
use uom::si::{
    angle::{degree, revolution},
    f32::Angle,
};

use super::{
    clamp_revolution::{clamp_revolution, scale_revolution_to_range, Clamping},
    tension_arm::TensionArm,
};

pub trait SpoolSpeedControllerTrait
where
    Self: Debug,
{
    fn get_speed(&mut self, nanoseconds: u64, tension_arm: &TensionArm) -> i32;
    fn reset(&mut self);
}

#[derive(Debug)]
pub struct LinearSpoolSpeedController {
    // current speed in steps/s
    speed: f32,
    max_speed: f32,
    dampening_controller: PidController,
}

impl LinearSpoolSpeedController {
    pub fn new(max_speed: f32) -> Self {
        Self {
            max_speed,
            speed: 0.0,
            dampening_controller: PidController::new(max_speed * 10e-9, 0.0, 0.0),
        }
    }
}

impl LinearSpoolSpeedController {
    fn speed_raw(&mut self, _nanoseconds: u64, tension_arm: &TensionArm) -> f32 {
        // calculate filament tension
        let tension_arm_min_degree: f32 = Angle::new::<degree>(10.0).get::<revolution>();
        let tension_arm_max_degree: f32 = Angle::new::<degree>(80.0).get::<revolution>();
        let tension_arm_angle = tension_arm.get_angle();
        let tension_arm_revolution = clamp_revolution(
            tension_arm_angle.get::<revolution>(),
            tension_arm_min_degree,
            tension_arm_max_degree,
        );

        match tension_arm_revolution.1 {
            Clamping::Min => return 0.0,
            Clamping::Max => return 0.0,
            _ => {}
        };

        let filament_tension = scale_revolution_to_range(
            tension_arm_revolution.0,
            tension_arm_min_degree,
            tension_arm_max_degree,
        );

        let filament_tension_inverted = 1.0 - filament_tension;

        // interpolate speed
        let speed = filament_tension_inverted * (self.max_speed - 0.0) + 0.0;

        // save speed
        return speed;
    }

    fn dampen_speed(&mut self, nanoseconds: u64, speed: f32) -> f32 {
        let error = speed - self.speed;
        let acceleration = self.dampening_controller.update(error, nanoseconds);
        let new_speed = self.speed + acceleration;
        return new_speed;
    }

    fn clamp_speed(&mut self, speed: f32) -> f32 {
        if speed < 0.0 {
            return 0.0;
        } else if speed > self.max_speed {
            return self.max_speed;
        } else {
            return speed;
        }
    }
}

impl SpoolSpeedControllerTrait for LinearSpoolSpeedController {
    fn get_speed(&mut self, _nanoseconds: u64, tension_arm: &TensionArm) -> i32 {
        let speed = self.speed_raw(_nanoseconds, tension_arm);
        let speed = self.dampen_speed(_nanoseconds, speed);
        let speed = self.clamp_speed(speed);

        self.speed = speed;
        return speed as i32;
    }

    fn reset(&mut self) {
        self.speed = 0.0;
        self.dampening_controller.reset();
    }
}
