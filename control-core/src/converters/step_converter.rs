use uom::si::{
    angle::revolution,
    angular_acceleration::radian_per_second_squared,
    angular_velocity::revolution_per_second,
    f32::{Angle, AngularAcceleration, AngularVelocity},
};

#[derive(Debug, Clone)]
pub struct StepConverter {
    steps_per_revolution: i16,
}

impl StepConverter {
    pub fn new(steps_per_revolution: i16) -> Self {
        Self {
            steps_per_revolution,
        }
    }

    /// Convert steps to angle
    pub fn steps_to_angle(&self, steps: f32) -> Angle {
        let revolutions = steps as f32 / self.steps_per_revolution as f32;
        Angle::new::<revolution>(revolutions)
    }

    /// Convert angle to steps
    pub fn angle_to_steps(&self, angle: Angle) -> f32 {
        let revolutions = angle.get::<revolution>();
        revolutions * self.steps_per_revolution as f32
    }

    /// Convert steps/second to angular velocity
    pub fn steps_to_angular_velocity(&self, steps: f32) -> AngularVelocity {
        let revolutions_per_second = steps as f32 / self.steps_per_revolution as f32;
        AngularVelocity::new::<revolution_per_second>(revolutions_per_second)
    }

    /// Convert angular velocity to steps/second
    pub fn angular_velocity_to_steps(&self, angular_velocity: AngularVelocity) -> f32 {
        let revolutions_per_second = angular_velocity.get::<revolution_per_second>();
        revolutions_per_second * self.steps_per_revolution as f32
    }

    /// Convert steps/second^2 to angular acceleration
    pub fn steps_to_angular_acceleration(&self, steps: f32) -> AngularAcceleration {
        let revolutions_per_second_squared = steps as f32 / self.steps_per_revolution as f32;
        let radians_per_second_squared =
            revolutions_per_second_squared * 2.0 * std::f32::consts::PI;
        AngularAcceleration::new::<radian_per_second_squared>(radians_per_second_squared)
    }

    /// Convert angular acceleration to steps/second^2
    pub fn angular_acceleration_to_steps(&self, angular_acceleration: AngularAcceleration) -> f32 {
        let radians_per_second_squared = angular_acceleration.get::<radian_per_second_squared>();
        let revolutions_per_second_squared =
            radians_per_second_squared / (2.0 * std::f32::consts::PI);
        revolutions_per_second_squared * self.steps_per_revolution as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f32::EPSILON;
    use uom::si::{
        angle::degree, angular_acceleration::degree_per_second_squared,
        angular_velocity::degree_per_second,
    };

    #[test]
    fn test_new() {
        let converter = StepConverter::new(200);
        assert_eq!(converter.steps_per_revolution, 200);
    }

    #[test]
    fn test_steps_to_angle() {
        let converter = StepConverter::new(200);

        // Test full revolution
        let angle = converter.steps_to_angle(200.0);
        assert_relative_eq!(angle.get::<revolution>(), 1.0, epsilon = EPSILON);
        assert_relative_eq!(angle.get::<degree>(), 360.0, epsilon = EPSILON);

        // Test half revolution
        let angle = converter.steps_to_angle(100.0);
        assert_relative_eq!(angle.get::<revolution>(), 0.5, epsilon = EPSILON);
        assert_relative_eq!(angle.get::<degree>(), 180.0, epsilon = EPSILON);

        // Test quarter revolution
        let angle = converter.steps_to_angle(50.0);
        assert_relative_eq!(angle.get::<revolution>(), 0.25, epsilon = EPSILON);
        assert_relative_eq!(angle.get::<degree>(), 90.0, epsilon = EPSILON);

        // Test with different steps_per_revolution
        let converter = StepConverter::new(400);
        let angle = converter.steps_to_angle(100.0);
        assert_relative_eq!(angle.get::<revolution>(), 0.25, epsilon = EPSILON);
    }

    #[test]
    fn test_angle_to_steps() {
        let converter = StepConverter::new(200);

        // Test full revolution
        let steps = converter.angle_to_steps(Angle::new::<revolution>(1.0));
        assert_relative_eq!(steps, 200.0, epsilon = EPSILON);

        // Test half revolution
        let steps = converter.angle_to_steps(Angle::new::<revolution>(0.5));
        assert_relative_eq!(steps, 100.0, epsilon = EPSILON);

        // Test using degrees
        let steps = converter.angle_to_steps(Angle::new::<degree>(90.0));
        assert_relative_eq!(steps, 50.0, epsilon = EPSILON);

        // Test with different steps_per_revolution
        let converter = StepConverter::new(400);
        let steps = converter.angle_to_steps(Angle::new::<degree>(90.0));
        assert_relative_eq!(steps, 100.0, epsilon = EPSILON);
    }

    #[test]
    fn test_steps_to_angular_velocity() {
        let converter = StepConverter::new(200);

        // Test one revolution per second
        let angular_velocity = converter.steps_to_angular_velocity(200.0);
        assert_relative_eq!(
            angular_velocity.get::<revolution_per_second>(),
            1.0,
            epsilon = EPSILON
        );
        assert_relative_eq!(
            angular_velocity.get::<degree_per_second>(),
            360.0,
            epsilon = EPSILON
        );

        // Test half revolution per second
        let angular_velocity = converter.steps_to_angular_velocity(100.0);
        assert_relative_eq!(
            angular_velocity.get::<revolution_per_second>(),
            0.5,
            epsilon = EPSILON
        );

        // Test with different steps_per_revolution
        let converter = StepConverter::new(400);
        let angular_velocity = converter.steps_to_angular_velocity(200.0);
        assert_relative_eq!(
            angular_velocity.get::<revolution_per_second>(),
            0.5,
            epsilon = EPSILON
        );
    }

    #[test]
    fn test_angular_velocity_to_steps() {
        let converter = StepConverter::new(200);

        // Test one revolution per second
        let steps =
            converter.angular_velocity_to_steps(AngularVelocity::new::<revolution_per_second>(1.0));
        assert_relative_eq!(steps, 200.0, epsilon = EPSILON);

        // Test using degrees per second
        let steps =
            converter.angular_velocity_to_steps(AngularVelocity::new::<degree_per_second>(180.0));
        assert_relative_eq!(steps, 100.0, epsilon = EPSILON);

        // Test with different steps_per_revolution
        let converter = StepConverter::new(400);
        let steps =
            converter.angular_velocity_to_steps(AngularVelocity::new::<revolution_per_second>(0.5));
        assert_relative_eq!(steps, 200.0, epsilon = EPSILON);
    }

    #[test]
    fn test_steps_to_angular_acceleration() {
        let converter = StepConverter::new(200);

        // Test one revolution per second squared
        let angular_acceleration = converter.steps_to_angular_acceleration(200.0);
        let expected_rps2 = 2.0 * std::f32::consts::PI; // 1 rev/s² = 2π rad/s²
        assert_relative_eq!(
            angular_acceleration.get::<radian_per_second_squared>(),
            expected_rps2,
            epsilon = EPSILON
        );

        // Test half revolution per second squared
        let angular_acceleration = converter.steps_to_angular_acceleration(100.0);
        assert_relative_eq!(
            angular_acceleration.get::<radian_per_second_squared>(),
            expected_rps2 / 2.0,
            epsilon = EPSILON
        );

        // Test with different steps_per_revolution
        let converter = StepConverter::new(400);
        let angular_acceleration = converter.steps_to_angular_acceleration(400.0);
        assert_relative_eq!(
            angular_acceleration.get::<radian_per_second_squared>(),
            expected_rps2,
            epsilon = EPSILON
        );
    }

    #[test]
    fn test_angular_acceleration_to_steps() {
        let converter = StepConverter::new(200);

        // Test one revolution per second squared (2π rad/s²)
        let rps2 = 2.0 * std::f32::consts::PI;
        let steps = converter.angular_acceleration_to_steps(AngularAcceleration::new::<
            radian_per_second_squared,
        >(rps2));
        assert_relative_eq!(steps, 200.0, epsilon = EPSILON);

        // Test using degrees per second squared
        let steps = converter.angular_acceleration_to_steps(AngularAcceleration::new::<
            degree_per_second_squared,
        >(180.0));
        assert_relative_eq!(steps, 100.0, epsilon = EPSILON);

        // Test with different steps_per_revolution
        let converter = StepConverter::new(400);
        let steps = converter.angular_acceleration_to_steps(AngularAcceleration::new::<
            radian_per_second_squared,
        >(rps2));
        assert_relative_eq!(steps, 400.0, epsilon = EPSILON);
    }

    #[test]
    fn test_roundtrip_conversions() {
        let converter = StepConverter::new(200);

        // Test angle roundtrip
        let original_steps = 123.0;
        let angle = converter.steps_to_angle(original_steps);
        let roundtrip_steps = converter.angle_to_steps(angle);
        assert_relative_eq!(original_steps, roundtrip_steps, epsilon = EPSILON);

        // Test angular velocity roundtrip
        let original_steps = 456.0;
        let velocity = converter.steps_to_angular_velocity(original_steps);
        let roundtrip_steps = converter.angular_velocity_to_steps(velocity);
        assert_relative_eq!(original_steps, roundtrip_steps, epsilon = EPSILON);

        // Test angular acceleration roundtrip
        let original_steps = 789.0;
        let acceleration = converter.steps_to_angular_acceleration(original_steps);
        let roundtrip_steps = converter.angular_acceleration_to_steps(acceleration);
        assert_relative_eq!(original_steps, roundtrip_steps, epsilon = EPSILON);
    }
}
