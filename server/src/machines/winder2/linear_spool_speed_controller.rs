use control_core::controllers::pid::PidController;
use uom::si::{
    angle::{degree, revolution},
    f32::Angle,
};

use super::{
    clamp_revolution::{clamp_revolution, scale_revolution_to_range, Clamping},
    spool_speed_controller::SpoolSpeedControllerTrait,
    tension_arm::TensionArm,
};

#[derive(Debug)]
pub struct LinearSpoolSpeedController {
    // current speed in steps/s
    speed: f32,
    min_speed: f32,
    max_speed: f32,
    enabled: bool,
    dampening_controller: PidController,
}

impl LinearSpoolSpeedController {
    pub fn new(min_speed: f32, max_speed: f32) -> Self {
        Self {
            min_speed,
            max_speed,
            speed: 0.0,
            enabled: false,
            dampening_controller: PidController::new(max_speed * 3e-8, 0.0, 0.0),
        }
    }
}

impl LinearSpoolSpeedController {
    fn speed_raw(&mut self, _nanoseconds: u64, tension_arm: &TensionArm) -> f32 {
        let min_speed = self.min_speed * 0.0;
        let max_speed = self.max_speed * 1.0;

        // calculate filament tension
        let tension_arm_min_degree: f32 = Angle::new::<degree>(20.0).get::<revolution>();
        let tension_arm_max_degree: f32 = Angle::new::<degree>(90.0).get::<revolution>();
        let tension_arm_angle = tension_arm.get_angle();
        let tension_arm_revolution = clamp_revolution(
            tension_arm_angle.get::<revolution>(),
            tension_arm_min_degree,
            tension_arm_max_degree,
        );

        match tension_arm_revolution.1 {
            Clamping::Min => return min_speed,
            Clamping::Max => return min_speed,
            _ => {}
        };

        let filament_tension = scale_revolution_to_range(
            tension_arm_revolution.0,
            tension_arm_min_degree,
            tension_arm_max_degree,
        );

        let filament_tension_inverted = 1.0 - filament_tension;

        // interpolate speed
        let speed = filament_tension_inverted * (max_speed - min_speed) + min_speed;

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
    fn get_speed(&mut self, nanoseconds: u64, tension_arm: &TensionArm) -> i32 {
        let speed = self.speed_raw(nanoseconds, tension_arm);
        let speed = match self.enabled {
            true => speed,
            false => 0.0,
        };
        let speed = self.dampen_speed(nanoseconds, speed);

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
        self.max_speed = max_speed;
    }

    fn set_min_speed(&mut self, min_speed: f32) {
        self.min_speed = min_speed;
    }

    fn get_max_speed(&self) -> f32 {
        self.max_speed
    }

    fn get_min_speed(&self) -> f32 {
        self.min_speed
    }
}
