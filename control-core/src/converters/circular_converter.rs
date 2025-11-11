//! # Circular Converter
//!
//! This module provides the `CircularConverter` which handles transformations between
//! linear and angular measurements for circular motion systems.
//!
//! ## Usage
//!
//! The `CircularConverter` is designed to work alongside other converters:
//! - Use `AngularStepConverter` for angular-to-step conversions
//! - Use `LinearStepConverter` for complete linear-to-step conversions (uses `CircularConverter` internally)
//! - Use `CircularConverter` directly when you need linear-to-angular conversions without steps
//!
//! ## Examples
//!
//! ```rust
//! use control_core::converters::circular_converter::CircularConverter;
//! use units::{f64::{Length, Velocity}, length::meter, velocity::meter_per_second};
//!
//! let converter = CircularConverter::from_radius(Length::new::<meter>(0.1));
//! let velocity = Velocity::new::<meter_per_second>(1.0);
//! let angular_velocity = converter.linear_to_angular_velocity(velocity);
//! ```

use units::{
    acceleration::meter_per_second_squared,
    angle::radian,
    angular_acceleration::radian_per_second_squared,
    angular_velocity::radian_per_second,
    f64::{Acceleration, Angle, AngularAcceleration, AngularVelocity, Length, Velocity},
    length::meter,
    velocity::meter_per_second,
};

/// A converter that handles transformations between linear and angular measurements
/// for circular motion. This includes conversions for position, velocity, acceleration,
/// and higher-order derivatives based on a circle's geometric properties.
#[derive(Debug, Clone)]
pub struct CircularConverter {
    radius: Length,
    circumference: Length,
    diameter: Length,
}

impl CircularConverter {
    /// Create a new converter from radius
    pub fn from_radius(radius: Length) -> Self {
        let diameter = radius * 2.0;
        let circumference =
            Length::new::<meter>(2.0 * std::f64::consts::PI * radius.get::<meter>());
        Self {
            radius,
            diameter,
            circumference,
        }
    }

    /// Create a new converter from diameter
    pub fn from_diameter(diameter: Length) -> Self {
        let radius = diameter / 2.0;
        Self::from_radius(radius)
    }

    /// Create a new converter from circumference
    pub fn from_circumference(circumference: Length) -> Self {
        let radius =
            Length::new::<meter>(circumference.get::<meter>() / (2.0 * std::f64::consts::PI));
        Self::from_radius(radius)
    }

    /// Get the radius
    pub fn radius(&self) -> Length {
        self.radius
    }

    /// Get the diameter
    pub fn diameter(&self) -> Length {
        self.diameter
    }

    /// Get the circumference
    pub fn circumference(&self) -> Length {
        self.circumference
    }

    // Linear to Angular Position Conversions

    /// Convert linear distance to angular position
    /// Formula: angle = distance / radius
    pub fn linear_to_angular_position(&self, distance: Length) -> Angle {
        let angle_rad = distance.get::<meter>() / self.radius.get::<meter>();
        Angle::new::<radian>(angle_rad)
    }

    /// Convert angular position to linear distance
    /// Formula: distance = angle * radius
    pub fn angular_to_linear_position(&self, angle: Angle) -> Length {
        let distance = angle.get::<radian>() * self.radius.get::<meter>();
        Length::new::<meter>(distance)
    }

    // Linear to Angular Velocity Conversions

    /// Convert linear velocity to angular velocity
    /// Formula: angular_velocity = linear_velocity / radius
    pub fn linear_to_angular_velocity(&self, velocity: Velocity) -> AngularVelocity {
        let angular_velocity_rad_per_s =
            velocity.get::<meter_per_second>() / self.radius.get::<meter>();
        AngularVelocity::new::<radian_per_second>(angular_velocity_rad_per_s)
    }

    /// Convert angular velocity to linear velocity
    /// Formula: linear_velocity = angular_velocity * radius
    pub fn angular_to_linear_velocity(&self, angular_velocity: AngularVelocity) -> Velocity {
        let velocity = angular_velocity.get::<radian_per_second>() * self.radius.get::<meter>();
        Velocity::new::<meter_per_second>(velocity)
    }

    // Linear to Angular Acceleration Conversions

    /// Convert linear acceleration to angular acceleration
    /// Formula: angular_acceleration = linear_acceleration / radius
    pub fn linear_to_angular_acceleration(
        &self,
        acceleration: Acceleration,
    ) -> AngularAcceleration {
        let angular_acceleration_rad_per_s2 =
            acceleration.get::<meter_per_second_squared>() / self.radius.get::<meter>();
        AngularAcceleration::new::<radian_per_second_squared>(angular_acceleration_rad_per_s2)
    }

    /// Convert angular acceleration to linear acceleration
    /// Formula: linear_acceleration = angular_acceleration * radius
    pub fn angular_to_linear_acceleration(
        &self,
        angular_acceleration: AngularAcceleration,
    ) -> Acceleration {
        let acceleration =
            angular_acceleration.get::<radian_per_second_squared>() * self.radius.get::<meter>();
        Acceleration::new::<meter_per_second_squared>(acceleration)
    }

    // Jerk Conversions (using f64 for now since UOM may not have jerk units)

    /// Convert linear jerk to angular jerk
    /// Formula: angular_jerk = linear_jerk / radius
    /// Units: rad/s³ = (m/s³) / m
    pub fn linear_to_angular_jerk(&self, linear_jerk: f64) -> f64 {
        linear_jerk / self.radius.get::<meter>()
    }

    /// Convert angular jerk to linear jerk
    /// Formula: linear_jerk = angular_jerk * radius
    /// Units: m/s³ = (rad/s³) * m
    pub fn angular_to_linear_jerk(&self, angular_jerk: f64) -> f64 {
        angular_jerk * self.radius.get::<meter>()
    }

    // Arc Length Calculations

    /// Calculate arc length from angle
    /// Formula: arc_length = angle * radius
    pub fn angle_to_arc_length(&self, angle: Angle) -> Length {
        self.angular_to_linear_position(angle)
    }

    /// Calculate angle from arc length
    /// Formula: angle = arc_length / radius
    pub fn arc_length_to_angle(&self, arc_length: Length) -> Angle {
        self.linear_to_angular_position(arc_length)
    }

    // Circumference-based calculations

    /// Convert linear distance to number of revolutions
    /// Formula: revolutions = distance / circumference
    pub fn linear_distance_to_revolutions(&self, distance: Length) -> f64 {
        distance.get::<meter>() / self.circumference.get::<meter>()
    }

    /// Convert number of revolutions to linear distance
    /// Formula: distance = revolutions * circumference
    pub fn revolutions_to_linear_distance(&self, revolutions: f64) -> Length {
        Length::new::<meter>(revolutions * self.circumference.get::<meter>())
    }

    /// Convert linear velocity to revolutions per second
    /// Formula: rps = velocity / circumference
    pub fn linear_velocity_to_rps(&self, velocity: Velocity) -> f64 {
        velocity.get::<meter_per_second>() / self.circumference.get::<meter>()
    }

    /// Convert revolutions per second to linear velocity
    /// Formula: velocity = rps * circumference
    pub fn rps_to_linear_velocity(&self, rps: f64) -> Velocity {
        Velocity::new::<meter_per_second>(rps * self.circumference.get::<meter>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use units::length::centimeter;

    #[test]
    fn test_constructors() {
        let radius = Length::new::<centimeter>(5.0);
        let diameter = Length::new::<centimeter>(10.0);
        let circumference = Length::new::<centimeter>(2.0 * std::f64::consts::PI * 5.0);

        let converter1 = CircularConverter::from_radius(radius);
        let converter2 = CircularConverter::from_diameter(diameter);
        let converter3 = CircularConverter::from_circumference(circumference);

        // All should have approximately the same values
        assert!((converter1.radius().get::<centimeter>() - 5.0).abs() < 1e-10);
        assert!((converter2.radius().get::<centimeter>() - 5.0).abs() < 1e-10);
        assert!((converter3.radius().get::<centimeter>() - 5.0).abs() < 1e-10);

        assert!((converter1.diameter().get::<centimeter>() - 10.0).abs() < 1e-10);
        assert!((converter2.diameter().get::<centimeter>() - 10.0).abs() < 1e-10);
        assert!((converter3.diameter().get::<centimeter>() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_position_conversions() {
        let converter = CircularConverter::from_radius(Length::new::<meter>(1.0));

        // Test linear to angular
        let distance = Length::new::<meter>(std::f64::consts::PI);
        let angle = converter.linear_to_angular_position(distance);
        assert!((angle.get::<radian>() - std::f64::consts::PI).abs() < 1e-10);

        // Test angular to linear
        let angle = Angle::new::<radian>(std::f64::consts::PI / 2.0);
        let distance = converter.angular_to_linear_position(angle);
        assert!((distance.get::<meter>() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_velocity_conversions() {
        let converter = CircularConverter::from_radius(Length::new::<meter>(0.5));

        // Test linear to angular
        let velocity = Velocity::new::<meter_per_second>(1.0);
        let angular_velocity = converter.linear_to_angular_velocity(velocity);
        assert!((angular_velocity.get::<radian_per_second>() - 2.0).abs() < 1e-10);

        // Test angular to linear
        let angular_velocity = AngularVelocity::new::<radian_per_second>(4.0);
        let velocity = converter.angular_to_linear_velocity(angular_velocity);
        assert!((velocity.get::<meter_per_second>() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_revolution_conversions() {
        let converter = CircularConverter::from_radius(Length::new::<meter>(1.0));

        // One full revolution should equal circumference
        let revolutions = 1.0;
        let distance = converter.revolutions_to_linear_distance(revolutions);
        assert!((distance.get::<meter>() - converter.circumference().get::<meter>()).abs() < 1e-10);

        // Converting back should give 1 revolution
        let calculated_revolutions = converter.linear_distance_to_revolutions(distance);
        assert!((calculated_revolutions - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_jerk_conversions() {
        let converter = CircularConverter::from_radius(Length::new::<meter>(2.0));

        let linear_jerk = 4.0; // m/s³
        let angular_jerk = converter.linear_to_angular_jerk(linear_jerk);
        assert!((angular_jerk - 2.0).abs() < 1e-10); // 4.0 / 2.0 = 2.0 rad/s³

        let calculated_linear_jerk = converter.angular_to_linear_jerk(angular_jerk);
        assert!((calculated_linear_jerk - linear_jerk).abs() < 1e-10);
    }
}
