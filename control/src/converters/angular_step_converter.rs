use qitech_lib::units::angle::revolution;
use qitech_lib::units::angular_acceleration::radian_per_second_squared;
use qitech_lib::units::angular_velocity::revolution_per_second;
use qitech_lib::units::f64::Angle;
use qitech_lib::units::f64::AngularAcceleration;
use qitech_lib::units::f64::AngularVelocity;

/// Converts between motor steps and angular measurements
///
/// This converter handles the transformation between motor steps and angular
/// measurements (angle, angular velocity, and angular acceleration) based on
/// the number of steps required for a full revolution.
#[derive(Debug, Clone)]
pub struct AngularStepConverter {
    /// Number of motor steps required for one complete revolution
    pub steps_per_revolution: i16,
}

impl AngularStepConverter {
    /// Create a new converter with the specified steps per revolution
    pub const fn new(steps_per_revolution: i16) -> Self {
        Self {
            steps_per_revolution,
        }
    }

    /// Convert steps to angle
    ///
    /// Formula: angle (in revolutions) = steps / steps_per_revolution
    pub fn steps_to_angle(&self, steps: f64) -> Angle {
        let revolutions = steps / self.steps_per_revolution as f64;
        Angle::new::<revolution>(revolutions)
    }

    /// Convert angle to steps
    ///
    /// Formula: steps = angle (in revolutions) * steps_per_revolution
    pub fn angle_to_steps(&self, angle: Angle) -> f64 {
        let revolutions = angle.get::<revolution>();
        revolutions * self.steps_per_revolution as f64
    }

    /// Convert steps/second to angular velocity
    ///
    /// Formula: angular velocity (in rev/s) = steps_per_second / steps_per_revolution
    pub fn steps_to_angular_velocity(&self, steps_per_second: f64) -> AngularVelocity {
        let revolutions_per_second = steps_per_second / self.steps_per_revolution as f64;
        AngularVelocity::new::<revolution_per_second>(revolutions_per_second)
    }

    /// Convert angular velocity to steps/second
    ///
    /// Formula: steps_per_second = angular velocity (in rev/s) * steps_per_revolution
    pub fn angular_velocity_to_steps(&self, angular_velocity: AngularVelocity) -> f64 {
        let revolutions_per_second = angular_velocity.get::<revolution_per_second>();
        revolutions_per_second * self.steps_per_revolution as f64
    }

    /// Convert steps/second² to angular acceleration
    ///
    /// Formula:
    /// - revolutions_per_second² = steps_per_second² / steps_per_revolution
    /// - angular acceleration (in rad/s²) = revolutions_per_second² * 2π
    pub fn steps_to_angular_acceleration(
        &self,
        steps_per_second_squared: f64,
    ) -> AngularAcceleration {
        let revolutions_per_second_squared =
            steps_per_second_squared / self.steps_per_revolution as f64;
        let radians_per_second_squared =
            revolutions_per_second_squared * 2.0 * std::f64::consts::PI;
        AngularAcceleration::new::<radian_per_second_squared>(radians_per_second_squared)
    }

    /// Convert angular acceleration to steps/second²
    ///
    /// Formula:
    /// - revolutions_per_second² = angular acceleration (in rad/s²) / 2π
    /// - steps_per_second² = revolutions_per_second² * steps_per_revolution
    pub fn angular_acceleration_to_steps(&self, angular_acceleration: AngularAcceleration) -> f64 {
        let radians_per_second_squared = angular_acceleration.get::<radian_per_second_squared>();
        let revolutions_per_second_squared =
            radians_per_second_squared / (2.0 * std::f64::consts::PI);
        revolutions_per_second_squared * self.steps_per_revolution as f64
    }
}
