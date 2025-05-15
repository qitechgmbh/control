use super::{
    clamp_revolution::{Clamping, clamp_revolution, scale_revolution_to_range},
    tension_arm::TensionArm,
};
use control_core::{
    controllers::linear_acceleration::LinearAngularAccelerationController,
    helpers::interpolation::{interpolate_exponential, scale},
    uom_extensions::angular_acceleration::revolutions_per_minute_per_second,
};
use std::time::Instant;
use uom::{
    ConstZero,
    si::{
        angle::{degree, revolution},
        angular_velocity::{radian_per_second, revolution_per_minute},
        f64::{Angle, AngularAcceleration, AngularVelocity},
    },
};

#[derive(Debug)]
pub struct SpoolSpeedController {
    /// Current speed in
    speed: AngularVelocity,
    /// Minimum speed in
    min_speed: AngularVelocity,
    /// Maximum speed in
    max_speed: AngularVelocity,
    /// Whether the speed controller is enabled or not
    enabled: bool,
    /// Linear acceleration controller to dampen speed change
    acceleration_controller: LinearAngularAccelerationController,
}

impl SpoolSpeedController {
    /// Parameters:
    /// - `min_speed`: Minimum
    /// - `max_speed`: Maximum
    /// - `acceleration`: Acceleration
    /// - `deceleration`: Deceleration (preferably negative)
    pub fn new(
        min_speed: AngularVelocity,
        max_speed: AngularVelocity,
        acceleration: AngularAcceleration,
        deceleration: AngularAcceleration,
    ) -> Self {
        Self {
            min_speed,
            max_speed,
            speed: AngularVelocity::ZERO,
            enabled: false,
            acceleration_controller: LinearAngularAccelerationController::new(
                acceleration,
                deceleration,
                AngularVelocity::ZERO,
            ),
        }
    }
}

impl SpoolSpeedController {
    /// Calculates the desired speed based on the tension arm angle.
    ///
    /// If the arm is over it's maximum angle, the speed is set to the minimum speed.
    /// If the arm is under it's minimum angle, the speed is set to the maximum speed.
    /// If the arm is within the range, the speed is interpolated between the minimum and maximum speed based on the tension arm angle.
    ///
    /// Parameters:
    /// - `t`: The current time.
    /// - `tension_arm`: A reference to the `TensionArm` instance that provides the angle of the tension arm.
    ///
    /// Returns:
    /// - speed
    fn speed_raw(&mut self, _t: Instant, tension_arm: &TensionArm) -> AngularVelocity {
        let min_speed = self.min_speed * 0.0;
        let max_speed = self.max_speed * 1.0;

        // calculate filament tension
        let tension_arm_min_degree: f64 = Angle::new::<degree>(20.0).get::<revolution>();
        let tension_arm_max_degree: f64 = Angle::new::<degree>(90.0).get::<revolution>();
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

        // use exponetial interpolation to make the speed change more sensitive in the lower range
        let filament_tension_exponential = interpolate_exponential(filament_tension_inverted, 2.0);

        // interpolate speed linear
        let speed = AngularVelocity::new::<radian_per_second>(scale(
            filament_tension_exponential,
            min_speed.get::<radian_per_second>(),
            max_speed.get::<radian_per_second>(),
        ));

        // save speed
        return speed;
    }

    /// Accelerates the speed using the acceleration controller.
    ///
    /// Parameters:
    /// - `speed`: The current speed
    /// - `t`: The current time.
    ///
    /// Returns:
    /// - The new speed after applying acceleration.
    fn accelerate_speed(&mut self, speed: AngularVelocity, t: Instant) -> AngularVelocity {
        self.acceleration_controller.update(speed, t)
    }

    /// Clamps the speed to the defined minimum and maximum speed.
    ///
    /// Parameters:
    /// - `speed`: The speed to be clamped.
    ///
    /// Returns:
    /// - The clamped speed, ensuring it is within the range of `min_speed` and `max_speed`.
    fn clamp_speed(&mut self, speed: AngularVelocity) -> AngularVelocity {
        if speed < self.min_speed {
            return AngularVelocity::ZERO;
        } else if speed > self.max_speed {
            return self.max_speed;
        } else {
            return speed;
        }
    }
}

impl SpoolSpeedController {
    pub fn get_speed(&mut self, t: Instant, tension_arm: &TensionArm) -> AngularVelocity {
        let speed = self.speed_raw(t, tension_arm);
        let speed = match self.enabled {
            true => speed,
            false => AngularVelocity::ZERO,
        };
        let speed = self.accelerate_speed(speed, t);

        // save speed before clamping or it will stay 0.0
        self.speed = speed;

        self.clamp_speed(speed)
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn reset(&mut self) {
        self.speed = AngularVelocity::ZERO;
        self.acceleration_controller.reset(AngularVelocity::ZERO);
    }

    fn update_acceleration(&mut self) {
        // Set acceleration to 1/10 of the range between min and max speed
        // The spool will accelerate from min to max speed in 10 seconds
        let range = self.max_speed - self.min_speed;
        let acceleration = AngularAcceleration::new::<revolutions_per_minute_per_second>(
            range.get::<revolution_per_minute>() / 10.0,
        );
        self.acceleration_controller.set_acceleration(acceleration);
        self.acceleration_controller.set_deceleration(-acceleration);
    }

    pub fn set_max_speed(&mut self, max_speed: AngularVelocity) {
        self.max_speed = max_speed;
        self.update_acceleration();
    }

    pub fn set_min_speed(&mut self, min_speed: AngularVelocity) {
        self.min_speed = min_speed;
        self.update_acceleration();
    }

    pub fn get_max_speed(&self) -> AngularVelocity {
        self.max_speed
    }

    pub fn get_min_speed(&self) -> AngularVelocity {
        self.min_speed
    }
}
