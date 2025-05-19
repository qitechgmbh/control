use uom::si::{
    acceleration::meter_per_second_squared,
    angle::revolution,
    angular_acceleration::radian_per_second_squared,
    angular_velocity::revolution_per_second,
    f64::{Acceleration, Angle, AngularAcceleration, AngularVelocity, Length, Velocity},
    length::meter,
    velocity::meter_per_second,
};

use super::step_converter::AngularStepConverter;

#[derive(Debug, Clone)]
pub struct LinearStepConverter {
    angular_step_converter: AngularStepConverter,
    radius: Length,
}

// Constructor and basic getters
impl LinearStepConverter {
    pub fn new(steps_per_revolution: i16, radius: Length) -> Self {
        Self {
            angular_step_converter: AngularStepConverter::new(steps_per_revolution),
            radius,
        }
    }

    /// Get the radius used by the converter
    pub fn radius(&self) -> Length {
        self.radius
    }

    /// Get the diameter of the system
    pub fn diameter(&self) -> Length {
        Length::new::<meter>(2.0 * self.radius.get::<meter>())
    }

    /// Get the circumference of the system
    pub fn circumference(&self) -> Length {
        Length::new::<meter>(2.0 * std::f64::consts::PI * self.radius.get::<meter>())
    }

    /// Get the steps per revolution
    pub fn steps_per_revolution(&self) -> i16 {
        self.angular_step_converter.steps_per_revolution
    }
}

// Linear to/from steps conversions
impl LinearStepConverter {
    /// Convert linear distance to steps
    pub fn distance_to_steps(&self, distance: Length) -> f64 {
        // Convert distance to angle: angle = distance/radius
        let circumference = 2.0 * std::f64::consts::PI * self.radius.get::<meter>();
        let revolutions = distance.get::<meter>() / circumference;
        let angle = Angle::new::<revolution>(revolutions);

        // Convert angle to steps
        self.angular_step_converter.angle_to_steps(angle)
    }

    /// Convert steps to linear distance
    pub fn steps_to_distance(&self, steps: f64) -> Length {
        // Convert steps to angle
        let angle = self.angular_step_converter.steps_to_angle(steps);

        // Convert angle to distance: distance = angle * radius
        let revolutions = angle.get::<revolution>();
        let circumference = 2.0 * std::f64::consts::PI * self.radius.get::<meter>();
        let distance = revolutions * circumference;

        Length::new::<meter>(distance)
    }

    /// Convert linear velocity to steps/second
    pub fn velocity_to_steps(&self, velocity: Velocity) -> f64 {
        // Convert linear velocity to angular velocity: ω = v/r
        let linear_velocity = velocity.get::<meter_per_second>();
        let angular_velocity_rps =
            linear_velocity / (2.0 * std::f64::consts::PI * self.radius.get::<meter>());
        let angular_velocity = AngularVelocity::new::<revolution_per_second>(angular_velocity_rps);

        // Convert angular velocity to steps/second
        self.angular_step_converter
            .angular_velocity_to_steps(angular_velocity)
    }

    /// Convert steps/second to linear velocity
    pub fn steps_to_velocity(&self, steps_per_second: f64) -> Velocity {
        // Convert steps/second to angular velocity
        let angular_velocity = self
            .angular_step_converter
            .steps_to_angular_velocity(steps_per_second);

        // Convert angular velocity to linear velocity: v = ω * r
        let angular_velocity_rps = angular_velocity.get::<revolution_per_second>();
        let linear_velocity =
            angular_velocity_rps * 2.0 * std::f64::consts::PI * self.radius.get::<meter>();

        Velocity::new::<meter_per_second>(linear_velocity)
    }

    /// Convert linear acceleration to steps/second²
    pub fn acceleration_to_steps(&self, acceleration: Acceleration) -> f64 {
        // Convert linear acceleration to angular acceleration: α = a/r
        let linear_acceleration = acceleration.get::<meter_per_second_squared>();
        // Calculate angular acceleration in radians per second squared
        let angular_acceleration_rad_per_s2 = linear_acceleration / self.radius.get::<meter>();
        let angular_acceleration =
            AngularAcceleration::new::<radian_per_second_squared>(angular_acceleration_rad_per_s2);

        // Convert angular acceleration to steps/second²
        self.angular_step_converter
            .angular_acceleration_to_steps(angular_acceleration)
    }

    /// Convert steps/second² to linear acceleration
    pub fn steps_to_acceleration(&self, steps_per_second_squared: f64) -> Acceleration {
        // Convert steps/second² to angular acceleration
        let angular_acceleration = self
            .angular_step_converter
            .steps_to_angular_acceleration(steps_per_second_squared);

        // Convert angular acceleration to linear acceleration: a = α * r
        let angular_acceleration_rad_per_s2 =
            angular_acceleration.get::<radian_per_second_squared>();
        let linear_acceleration = angular_acceleration_rad_per_s2 * self.radius.get::<meter>();

        Acceleration::new::<meter_per_second_squared>(linear_acceleration)
    }
}

// Linear to/from angular conversions
impl LinearStepConverter {
    /// Convert linear distance to angle
    pub fn distance_to_angle(&self, distance: Length) -> Angle {
        let circumference = 2.0 * std::f64::consts::PI * self.radius.get::<meter>();
        let revolutions = distance.get::<meter>() / circumference;
        Angle::new::<revolution>(revolutions)
    }

    /// Convert angle to linear distance
    pub fn angle_to_distance(&self, angle: Angle) -> Length {
        let revolutions = angle.get::<revolution>();
        let circumference = 2.0 * std::f64::consts::PI * self.radius.get::<meter>();
        let distance = revolutions * circumference;
        Length::new::<meter>(distance)
    }

    /// Convert linear velocity to angular velocity
    pub fn velocity_to_angular_velocity(&self, velocity: Velocity) -> AngularVelocity {
        let linear_velocity = velocity.get::<meter_per_second>();
        let angular_velocity_rps =
            linear_velocity / (2.0 * std::f64::consts::PI * self.radius.get::<meter>());
        AngularVelocity::new::<revolution_per_second>(angular_velocity_rps)
    }

    /// Convert angular velocity to linear velocity
    pub fn angular_velocity_to_velocity(&self, angular_velocity: AngularVelocity) -> Velocity {
        let angular_velocity_rps = angular_velocity.get::<revolution_per_second>();
        let linear_velocity =
            angular_velocity_rps * 2.0 * std::f64::consts::PI * self.radius.get::<meter>();
        Velocity::new::<meter_per_second>(linear_velocity)
    }

    /// Convert linear acceleration to angular acceleration
    pub fn acceleration_to_angular_acceleration(
        &self,
        acceleration: Acceleration,
    ) -> AngularAcceleration {
        let linear_acceleration = acceleration.get::<meter_per_second_squared>();
        let angular_acceleration_rad_per_s2 = linear_acceleration / self.radius.get::<meter>();
        AngularAcceleration::new::<radian_per_second_squared>(angular_acceleration_rad_per_s2)
    }

    /// Convert angular acceleration to linear acceleration
    pub fn angular_acceleration_to_acceleration(
        &self,
        angular_acceleration: AngularAcceleration,
    ) -> Acceleration {
        let angular_acceleration_rad_per_s2 =
            angular_acceleration.get::<radian_per_second_squared>();
        let linear_acceleration = angular_acceleration_rad_per_s2 * self.radius.get::<meter>();
        Acceleration::new::<meter_per_second_squared>(linear_acceleration)
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
        self.angular_step_converter.steps_to_angular_acceleration(steps)
    }

    /// Convert angular acceleration to steps/second²
    pub fn angular_acceleration_to_steps(&self, angular_acceleration: AngularAcceleration) -> f64 {
        self.angular_step_converter
            .angular_acceleration_to_steps(angular_acceleration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::EPSILON;

    #[test]
    fn test_new() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));
        assert_eq!(converter.steps_per_revolution(), 200);
        assert_eq!(converter.radius().get::<meter>(), 0.1);
    }

    #[test]
    fn test_diameter_and_circumference() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));

        // Test diameter
        let diameter = converter.diameter();
        assert_relative_eq!(diameter.get::<meter>(), 0.2, epsilon = EPSILON);

        // Test circumference
        let circumference = converter.circumference();
        let expected_circumference = 2.0 * std::f64::consts::PI * 0.1;
        assert_relative_eq!(
            circumference.get::<meter>(),
            expected_circumference,
            epsilon = EPSILON
        );
    }

    // Tests for linear <-> steps conversions
    #[test]
    fn test_distance_to_steps() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));
        let circumference = 2.0 * std::f64::consts::PI * 0.1; // 0.628m

        // One full revolution
        let steps = converter.distance_to_steps(Length::new::<meter>(circumference));
        assert_relative_eq!(steps, 200.0, epsilon = EPSILON);

        // Half revolution
        let steps = converter.distance_to_steps(Length::new::<meter>(circumference / 2.0));
        assert_relative_eq!(steps, 100.0, epsilon = EPSILON);
    }

    #[test]
    fn test_steps_to_distance() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));
        let circumference = 2.0 * std::f64::consts::PI * 0.1; // 0.628m

        // One full revolution
        let distance = converter.steps_to_distance(200.0);
        assert_relative_eq!(distance.get::<meter>(), circumference, epsilon = EPSILON);

        // Half revolution
        let distance = converter.steps_to_distance(100.0);
        assert_relative_eq!(
            distance.get::<meter>(),
            circumference / 2.0,
            epsilon = EPSILON
        );
    }

    #[test]
    fn test_velocity_to_steps() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));
        let circumference = 2.0 * std::f64::consts::PI * 0.1; // 0.628m

        // Linear velocity equivalent to one revolution per second
        let velocity = Velocity::new::<meter_per_second>(circumference);
        let steps_per_second = converter.velocity_to_steps(velocity);
        assert_relative_eq!(steps_per_second, 200.0, epsilon = EPSILON);

        // Half revolution per second
        let velocity = Velocity::new::<meter_per_second>(circumference / 2.0);
        let steps_per_second = converter.velocity_to_steps(velocity);
        assert_relative_eq!(steps_per_second, 100.0, epsilon = EPSILON);
    }

    #[test]
    fn test_steps_to_velocity() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));
        let circumference = 2.0 * std::f64::consts::PI * 0.1; // 0.628m

        // Steps for one revolution per second
        let velocity = converter.steps_to_velocity(200.0);
        assert_relative_eq!(
            velocity.get::<meter_per_second>(),
            circumference,
            epsilon = EPSILON
        );

        // Half revolution per second
        let velocity = converter.steps_to_velocity(100.0);
        assert_relative_eq!(
            velocity.get::<meter_per_second>(),
            circumference / 2.0,
            epsilon = EPSILON
        );
    }

    #[test]
    fn test_acceleration_to_steps() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));

        // Create a linear acceleration of 1 m/s²
        // This will result in angular acceleration of a/r = 1/0.1 = 10 rad/s²
        let acceleration = Acceleration::new::<meter_per_second_squared>(1.0);
        let steps_per_second_squared = converter.acceleration_to_steps(acceleration);

        // Expected steps for 10 rad/s² angular acceleration:
        // 10 rad/s² * (1 rev/2π rad) * 200 steps/rev = 318.31 steps/s²
        let expected_steps = 200.0 * 10.0 / (2.0 * std::f64::consts::PI);
        assert_relative_eq!(
            steps_per_second_squared,
            expected_steps,
            epsilon = EPSILON * 10.0
        );
    }

    #[test]
    fn test_steps_to_acceleration() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));

        // Steps for one revolution per second squared
        let acceleration = converter.steps_to_acceleration(200.0);
        let expected = 2.0 * std::f64::consts::PI * 0.1; // rad/s² * radius
        assert_relative_eq!(
            acceleration.get::<meter_per_second_squared>(),
            expected,
            epsilon = EPSILON
        );
    }

    // Tests for linear <-> angular conversions
    #[test]
    fn test_distance_to_angle() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));
        let circumference = 2.0 * std::f64::consts::PI * 0.1; // 0.628m

        // Full circumference = 1 revolution
        let angle = converter.distance_to_angle(Length::new::<meter>(circumference));
        assert_relative_eq!(angle.get::<revolution>(), 1.0, epsilon = EPSILON);

        // Half circumference = 0.5 revolution
        let angle = converter.distance_to_angle(Length::new::<meter>(circumference / 2.0));
        assert_relative_eq!(angle.get::<revolution>(), 0.5, epsilon = EPSILON);
    }

    #[test]
    fn test_angle_to_distance() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));
        let circumference = 2.0 * std::f64::consts::PI * 0.1; // 0.628m

        // 1 revolution = full circumference
        let distance = converter.angle_to_distance(Angle::new::<revolution>(1.0));
        assert_relative_eq!(distance.get::<meter>(), circumference, epsilon = EPSILON);

        // 0.5 revolution = half circumference
        let distance = converter.angle_to_distance(Angle::new::<revolution>(0.5));
        assert_relative_eq!(
            distance.get::<meter>(),
            circumference / 2.0,
            epsilon = EPSILON
        );
    }

    #[test]
    fn test_velocity_to_angular_velocity() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));
        let circumference = 2.0 * std::f64::consts::PI * 0.1; // 0.628m

        // Linear velocity of full circumference per second = 1 revolution per second
        let angular_velocity = converter
            .velocity_to_angular_velocity(Velocity::new::<meter_per_second>(circumference));
        assert_relative_eq!(
            angular_velocity.get::<revolution_per_second>(),
            1.0,
            epsilon = EPSILON
        );

        // Half circumference per second = 0.5 revolution per second
        let angular_velocity =
            converter.velocity_to_angular_velocity(Velocity::new::<meter_per_second>(
                circumference / 2.0,
            ));
        assert_relative_eq!(
            angular_velocity.get::<revolution_per_second>(),
            0.5,
            epsilon = EPSILON
        );
    }

    #[test]
    fn test_angular_velocity_to_velocity() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));
        let circumference = 2.0 * std::f64::consts::PI * 0.1; // 0.628m

        // 1 revolution per second = linear velocity of full circumference per second
        let velocity = converter
            .angular_velocity_to_velocity(AngularVelocity::new::<revolution_per_second>(1.0));
        assert_relative_eq!(
            velocity.get::<meter_per_second>(),
            circumference,
            epsilon = EPSILON
        );

        // 0.5 revolution per second = half circumference per second
        let velocity = converter
            .angular_velocity_to_velocity(AngularVelocity::new::<revolution_per_second>(0.5));
        assert_relative_eq!(
            velocity.get::<meter_per_second>(),
            circumference / 2.0,
            epsilon = EPSILON
        );
    }

    #[test]
    fn test_acceleration_to_angular_acceleration() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));

        // Linear acceleration of 1 m/s² = angular acceleration of 10 rad/s²
        let angular_accel = converter.acceleration_to_angular_acceleration(Acceleration::new::<
            meter_per_second_squared,
        >(1.0));
        assert_relative_eq!(
            angular_accel.get::<radian_per_second_squared>(),
            10.0, // a/r = 1/0.1 = 10
            epsilon = EPSILON
        );
    }

    #[test]
    fn test_angular_acceleration_to_acceleration() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));

        // Angular acceleration of 10 rad/s² = linear acceleration of 1 m/s²
        let acceleration =
            converter.angular_acceleration_to_acceleration(AngularAcceleration::new::<
                radian_per_second_squared,
            >(10.0));
        assert_relative_eq!(
            acceleration.get::<meter_per_second_squared>(),
            1.0, // α*r = 10*0.1 = 1
            epsilon = EPSILON
        );
    }

    // Tests for forwarded angular <-> steps methods
    #[test]
    fn test_forwarded_steps_to_angle() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));

        // 200 steps = 1 revolution
        let angle = converter.steps_to_angle(200.0);
        assert_relative_eq!(angle.get::<revolution>(), 1.0, epsilon = EPSILON);

        // 100 steps = 0.5 revolution
        let angle = converter.steps_to_angle(100.0);
        assert_relative_eq!(angle.get::<revolution>(), 0.5, epsilon = EPSILON);
    }

    #[test]
    fn test_forwarded_angle_to_steps() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));

        // 1 revolution = 200 steps
        let steps = converter.angle_to_steps(Angle::new::<revolution>(1.0));
        assert_relative_eq!(steps, 200.0, epsilon = EPSILON);

        // 0.5 revolution = 100 steps
        let steps = converter.angle_to_steps(Angle::new::<revolution>(0.5));
        assert_relative_eq!(steps, 100.0, epsilon = EPSILON);
    }

    #[test]
    fn test_roundtrip_conversions() {
        let converter = LinearStepConverter::new(200, Length::new::<meter>(0.1));

        // Test linear distance to steps roundtrip
        let original_distance = Length::new::<meter>(0.456);
        let steps = converter.distance_to_steps(original_distance);
        let roundtrip_distance = converter.steps_to_distance(steps);
        assert_relative_eq!(
            original_distance.get::<meter>(),
            roundtrip_distance.get::<meter>(),
            epsilon = EPSILON
        );

        // Test linear velocity to steps roundtrip
        let original_velocity = Velocity::new::<meter_per_second>(0.789);
        let steps_per_second = converter.velocity_to_steps(original_velocity);
        let roundtrip_velocity = converter.steps_to_velocity(steps_per_second);
        assert_relative_eq!(
            original_velocity.get::<meter_per_second>(),
            roundtrip_velocity.get::<meter_per_second>(),
            epsilon = EPSILON
        );

        // Test linear distance to angle roundtrip
        let original_distance = Length::new::<meter>(0.123);
        let angle = converter.distance_to_angle(original_distance);
        let roundtrip_distance = converter.angle_to_distance(angle);
        assert_relative_eq!(
            original_distance.get::<meter>(),
            roundtrip_distance.get::<meter>(),
            epsilon = EPSILON
        );

        // Test angle to steps roundtrip
        let original_angle = Angle::new::<revolution>(0.75);
        let steps = converter.angle_to_steps(original_angle);
        let roundtrip_angle = converter.steps_to_angle(steps);
        assert_relative_eq!(
            original_angle.get::<revolution>(),
            roundtrip_angle.get::<revolution>(),
            epsilon = EPSILON
        );
    }
}
