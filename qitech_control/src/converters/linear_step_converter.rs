use qitech_lib::units::angle::revolution;
use qitech_lib::units::angular_velocity::revolution_per_second;
use qitech_lib::units::f64::Acceleration;
use qitech_lib::units::f64::Angle;
use qitech_lib::units::f64::AngularAcceleration;
use qitech_lib::units::f64::AngularVelocity;
use qitech_lib::units::f64::Length;
use qitech_lib::units::f64::Velocity;

use super::angular_step_converter::AngularStepConverter;
use super::circular_converter::CircularConverter;

/// Converts between linear measurements and motor steps
///
/// This converter handles the transformation between linear motion (distance, velocity,
/// acceleration) and motor steps, using the relationship between linear and rotary motion
/// through a wheel or pulley of a specific radius.
#[derive(Debug, Clone)]
pub struct LinearStepConverter {
    angular_step_converter: AngularStepConverter,
    circular_converter: CircularConverter,
}

// Constructor and basic getters
impl LinearStepConverter {
    /// Create a new converter from radius and steps per revolution
    pub fn from_radius(steps_per_revolution: i16, radius: Length) -> Self {
        Self {
            angular_step_converter: AngularStepConverter::new(steps_per_revolution),
            circular_converter: CircularConverter::from_radius(radius),
        }
    }

    /// Create a new converter from diameter and steps per revolution
    pub fn from_diameter(steps_per_revolution: i16, diameter: Length) -> Self {
        Self {
            angular_step_converter: AngularStepConverter::new(steps_per_revolution),
            circular_converter: CircularConverter::from_diameter(diameter),
        }
    }

    /// Create a new converter from circumference and steps per revolution
    pub fn from_circumference(steps_per_revolution: i16, circumference: Length) -> Self {
        Self {
            angular_step_converter: AngularStepConverter::new(steps_per_revolution),
            circular_converter: CircularConverter::from_circumference(circumference),
        }
    }

    /// Get the radius used by the converter
    pub fn radius(&self) -> Length {
        self.circular_converter.radius()
    }

    /// Get the diameter of the system
    pub fn diameter(&self) -> Length {
        self.circular_converter.diameter()
    }

    /// Get the circumference of the system
    pub fn circumference(&self) -> Length {
        self.circular_converter.circumference()
    }

    /// Get the steps per revolution
    pub const fn steps_per_revolution(&self) -> i16 {
        self.angular_step_converter.steps_per_revolution
    }
}

// Linear to/from steps conversions
impl LinearStepConverter {
    /// Convert linear distance to steps
    ///
    /// Formula: steps = (distance / circumference) * steps_per_revolution
    pub fn distance_to_steps(&self, distance: Length) -> f64 {
        // Convert distance to revolutions using CircularConverter
        let revolutions = self
            .circular_converter
            .linear_distance_to_revolutions(distance);
        let angle = Angle::new::<revolution>(revolutions);

        // Convert angle to steps
        self.angular_step_converter.angle_to_steps(angle)
    }

    /// Convert steps to linear distance
    ///
    /// Formula: distance = (steps / steps_per_revolution) * circumference
    pub fn steps_to_distance(&self, steps: f64) -> Length {
        // Convert steps to angle
        let angle = self.angular_step_converter.steps_to_angle(steps);

        // Convert angle to distance using CircularConverter
        let revolutions = angle.get::<revolution>();
        self.circular_converter
            .revolutions_to_linear_distance(revolutions)
    }

    /// Convert linear velocity to steps/second
    ///
    /// Formula: steps/second = (velocity / circumference) * steps_per_revolution
    pub fn velocity_to_steps(&self, velocity: Velocity) -> f64 {
        // Convert linear velocity to revolutions per second using CircularConverter
        let rps = self.circular_converter.linear_velocity_to_rps(velocity);
        let angular_velocity = AngularVelocity::new::<revolution_per_second>(rps);

        // Convert angular velocity to steps/second
        self.angular_step_converter
            .angular_velocity_to_steps(angular_velocity)
    }

    /// Convert steps/second to linear velocity
    ///
    /// Formula: velocity = (steps_per_second / steps_per_revolution) * circumference
    pub fn steps_to_velocity(&self, steps_per_second: f64) -> Velocity {
        // Convert steps/second to angular velocity
        let angular_velocity = self
            .angular_step_converter
            .steps_to_angular_velocity(steps_per_second);

        // Convert angular velocity to linear velocity using CircularConverter
        let rps = angular_velocity.get::<revolution_per_second>();
        self.circular_converter.rps_to_linear_velocity(rps)
    }

    /// Convert linear acceleration to steps/second²
    ///
    /// Formula: steps/second² = (acceleration / radius) * (steps_per_revolution / (2π))
    pub fn acceleration_to_steps(&self, acceleration: Acceleration) -> f64 {
        // Convert linear acceleration to angular acceleration using CircularConverter
        let angular_acceleration = self
            .circular_converter
            .linear_to_angular_acceleration(acceleration);

        // Convert angular acceleration to steps/second²
        self.angular_step_converter
            .angular_acceleration_to_steps(angular_acceleration)
    }

    /// Convert steps/second² to linear acceleration
    ///
    /// Formula: acceleration = (steps_per_second² / steps_per_revolution) * (2π) * radius
    pub fn steps_to_acceleration(&self, steps_per_second_squared: f64) -> Acceleration {
        // Convert steps/second² to angular acceleration
        let angular_acceleration = self
            .angular_step_converter
            .steps_to_angular_acceleration(steps_per_second_squared);

        // Convert angular acceleration to linear acceleration using CircularConverter
        self.circular_converter
            .angular_to_linear_acceleration(angular_acceleration)
    }
}

// Linear to/from angular conversions
impl LinearStepConverter {
    /// Convert linear distance to angle
    ///
    /// Formula: angle (in revolutions) = distance / circumference
    pub fn distance_to_angle(&self, distance: Length) -> Angle {
        let revolutions = self
            .circular_converter
            .linear_distance_to_revolutions(distance);
        Angle::new::<revolution>(revolutions)
    }

    /// Convert angle to linear distance
    ///
    /// Formula: distance = angle (in revolutions) * circumference
    pub fn angle_to_distance(&self, angle: Angle) -> Length {
        let revolutions = angle.get::<revolution>();
        self.circular_converter
            .revolutions_to_linear_distance(revolutions)
    }

    /// Convert linear velocity to angular velocity
    ///
    /// Formula: angular velocity (in rev/s) = velocity / circumference
    pub fn velocity_to_angular_velocity(&self, velocity: Velocity) -> AngularVelocity {
        let rps = self.circular_converter.linear_velocity_to_rps(velocity);
        AngularVelocity::new::<revolution_per_second>(rps)
    }

    /// Convert angular velocity to linear velocity
    ///
    /// Formula: velocity = angular velocity (in rev/s) * circumference
    pub fn angular_velocity_to_velocity(&self, angular_velocity: AngularVelocity) -> Velocity {
        let rps = angular_velocity.get::<revolution_per_second>();
        self.circular_converter.rps_to_linear_velocity(rps)
    }

    /// Convert linear acceleration to angular acceleration
    ///
    /// Formula: angular acceleration (in rad/s²) = acceleration / radius
    pub fn acceleration_to_angular_acceleration(
        &self,
        acceleration: Acceleration,
    ) -> AngularAcceleration {
        self.circular_converter
            .linear_to_angular_acceleration(acceleration)
    }

    /// Convert angular acceleration to linear acceleration
    ///
    /// Formula: acceleration = angular acceleration (in rad/s²) * radius
    pub fn angular_acceleration_to_acceleration(
        &self,
        angular_acceleration: AngularAcceleration,
    ) -> Acceleration {
        self.circular_converter
            .angular_to_linear_acceleration(angular_acceleration)
    }
}

// Forward angular to/from steps conversions from StepConverter
impl LinearStepConverter {
    /// Convert steps to angle
    pub fn steps_to_angle(&self, steps: f64) -> Angle {
        self.angular_step_converter.steps_to_angle(steps)
    }

    /// Convert angle to steps
    pub fn angle_to_steps(&self, angle: Angle) -> f64 {
        self.angular_step_converter.angle_to_steps(angle)
    }

    /// Convert steps/second to angular velocity
    pub fn steps_to_angular_velocity(&self, steps: f64) -> AngularVelocity {
        self.angular_step_converter.steps_to_angular_velocity(steps)
    }

    /// Convert angular velocity to steps/second
    pub fn angular_velocity_to_steps(&self, angular_velocity: AngularVelocity) -> f64 {
        self.angular_step_converter
            .angular_velocity_to_steps(angular_velocity)
    }

    /// Convert steps/second² to angular acceleration
    pub fn steps_to_angular_acceleration(&self, steps: f64) -> AngularAcceleration {
        self.angular_step_converter
            .steps_to_angular_acceleration(steps)
    }

    /// Convert angular acceleration to steps/second²
    pub fn angular_acceleration_to_steps(&self, angular_acceleration: AngularAcceleration) -> f64 {
        self.angular_step_converter
            .angular_acceleration_to_steps(angular_acceleration)
    }
}
