use std::time::Instant;

use uom::si::{
    acceleration::meter_per_second_squared,
    angular_acceleration::radian_per_second_squared,
    angular_velocity::radian_per_second,
    f64::{Acceleration, AngularAcceleration, AngularVelocity, Velocity},
    velocity::meter_per_second,
};

#[derive(Debug)]
pub struct LinearAccelerationBaseController {
    /// Maximum acceleration in units per second (positive value)
    acceleration: f64,

    /// Maximum deceleration in units per second (negative value)
    deceleration: f64,

    /// Calculated speed at the last update
    speed: f64,

    /// Last update time
    last_t: Option<Instant>,
}

impl LinearAccelerationBaseController {
    pub fn new(acceleration: f64, deceleration: f64, initial_speed: f64) -> Self {
        Self {
            acceleration,
            deceleration,
            speed: initial_speed,
            last_t: None,
        }
    }

    pub fn update(&mut self, target_speed: f64, t: Instant) -> f64 {
        // Calculate time delta
        let dt = match self.last_t {
            Some(last) => {
                let duration = t.duration_since(last);
                duration.as_secs_f64()
            }
            None => 0.0, // First update, no acceleration applied
        };

        // Update the last update time
        self.last_t = Some(t);

        // Need to accelerate
        if self.speed < target_speed {
            let speed_change = self.acceleration * dt;
            let new_speed = (self.speed + speed_change).min(target_speed);
            self.speed = new_speed;
            return new_speed;
        }

        // Need to decelerate (deceleration is negative)
        if self.speed > target_speed {
            let speed_change = self.deceleration * dt; // This will be negative
            let new_speed = (self.speed + speed_change).max(target_speed);
            self.speed = new_speed;
            return new_speed;
        }

        self.speed
    }

    pub fn reset(&mut self, initial_speed: f64) {
        self.speed = initial_speed;
        self.last_t = None; // Reset the last update time
    }

    pub fn set_acceleration(&mut self, acceleration: f64) {
        self.acceleration = acceleration;
    }

    pub fn set_deceleration(&mut self, deceleration: f64) {
        self.deceleration = deceleration;
    }
}

/// [`LinearAngularAccelerationController`] wraps [`LinearAccelerationController`]
/// to handle angular velocities and accelerations.
#[derive(Debug)]
pub struct LinearAngularAccelerationController {
    pub controller: LinearAccelerationBaseController,
}

impl LinearAngularAccelerationController {
    pub fn new(
        acceleration: AngularAcceleration,
        deceleration: AngularAcceleration,
        initial_speed: AngularVelocity,
    ) -> Self {
        Self {
            controller: LinearAccelerationBaseController::new(
                acceleration.get::<radian_per_second_squared>(),
                deceleration.get::<radian_per_second_squared>(),
                initial_speed.get::<radian_per_second>(),
            ),
        }
    }

    pub fn update(&mut self, target_speed: AngularVelocity, t: Instant) -> AngularVelocity {
        let target_speed = target_speed.get::<radian_per_second>();
        let new_speed = self.controller.update(target_speed, t);
        return AngularVelocity::new::<radian_per_second>(new_speed);
    }

    pub fn reset(&mut self, initial_speed: AngularVelocity) {
        let initial_speed = initial_speed.get::<radian_per_second>();
        self.controller.reset(initial_speed);
    }

    pub fn set_acceleration(&mut self, acceleration: AngularAcceleration) {
        self.controller
            .set_acceleration(acceleration.get::<radian_per_second_squared>());
    }
    pub fn set_deceleration(&mut self, deceleration: AngularAcceleration) {
        self.controller
            .set_deceleration(deceleration.get::<radian_per_second_squared>());
    }
}

/// [`LinearAccelerationController`] wraps [`LinearF64AccelerationController`]
/// to handle linear velocities and accelerations.
#[derive(Debug)]
pub struct LinearAccelerationController {
    pub controller: LinearAccelerationBaseController,
}

impl LinearAccelerationController {
    pub fn new(
        acceleration: Acceleration,
        deceleration: Acceleration,
        initial_speed: Velocity,
    ) -> Self {
        Self {
            controller: LinearAccelerationBaseController::new(
                acceleration.get::<meter_per_second_squared>(),
                deceleration.get::<meter_per_second_squared>(),
                initial_speed.get::<meter_per_second>(),
            ),
        }
    }
    pub fn update(&mut self, target_speed: Velocity, t: Instant) -> Velocity {
        let target_speed = target_speed.get::<meter_per_second>();
        let new_speed = self.controller.update(target_speed, t);
        return Velocity::new::<meter_per_second>(new_speed);
    }
    pub fn reset(&mut self, initial_speed: Velocity) {
        let initial_speed = initial_speed.get::<meter_per_second>();
        self.controller.reset(initial_speed);
    }
    pub fn set_acceleration(&mut self, acceleration: Acceleration) {
        self.controller
            .set_acceleration(acceleration.get::<meter_per_second_squared>());
    }
    pub fn set_deceleration(&mut self, deceleration: Acceleration) {
        self.controller
            .set_deceleration(deceleration.get::<meter_per_second_squared>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::time::Duration;

    const EPSILON: f64 = 0.001;

    // Custom function to create a "future" instant for testing
    fn future_instant(now: Instant, seconds: f64) -> Instant {
        now + Duration::from_secs_f64(seconds)
    }

    #[test]
    fn test_initialization() {
        let controller = LinearAccelerationBaseController::new(10.0, -15.0, 5.0);
        assert_relative_eq!(controller.acceleration, 10.0, epsilon = EPSILON);
        assert_relative_eq!(controller.deceleration, -15.0, epsilon = EPSILON);
        assert_relative_eq!(controller.speed, 5.0, epsilon = EPSILON);
        assert!(controller.last_t.is_none());
    }

    #[test]
    fn test_first_update() {
        let mut controller = LinearAccelerationBaseController::new(10.0, -15.0, 5.0);
        let now = Instant::now();

        let speed = controller.update(5.0, now);

        assert_relative_eq!(speed, 5.0, epsilon = EPSILON);
        assert_relative_eq!(controller.speed, 5.0, epsilon = EPSILON);
        assert_eq!(controller.last_t.unwrap(), now);
    }

    #[test]
    fn test_acceleration() {
        let mut controller = LinearAccelerationBaseController::new(10.0, -15.0, 0.0);
        let t1 = Instant::now();
        controller.update(0.0, t1); // Initialize last_t

        // Use 0.1 seconds as our simulated time difference
        let dt = 0.1;
        let t2 = future_instant(t1, dt);

        let expected_speed = (10.0 * dt).min(5.0); // Should accelerate by max_acceleration * dt, capped at target

        let actual_speed = controller.update(5.0, t2);

        assert!(actual_speed > 0.0);
        assert!(actual_speed <= 5.0);
        assert_relative_eq!(actual_speed, expected_speed, epsilon = EPSILON);
    }

    #[test]
    fn test_deceleration() {
        let mut controller = LinearAccelerationBaseController::new(10.0, -15.0, 10.0);
        let t1 = Instant::now();
        controller.update(10.0, t1); // Initialize last_t

        // Use 0.1 seconds as our simulated time difference
        let dt = 0.1;
        let t2 = future_instant(t1, dt);

        let expected_speed = (10.0 + (-15.0 * dt)).max(5.0); // Should decelerate by max_deceleration * dt, floored at target

        let actual_speed = controller.update(5.0, t2);

        assert!(actual_speed < 10.0);
        assert!(actual_speed >= 5.0);
        assert_relative_eq!(actual_speed, expected_speed, epsilon = EPSILON);
    }

    #[test]
    fn test_constant_speed() {
        let mut controller = LinearAccelerationBaseController::new(10.0, -15.0, 7.0);
        let t1 = Instant::now();
        controller.update(7.0, t1); // Initialize last_t

        let t2 = future_instant(t1, 0.1);

        let speed = controller.update(7.0, t2);

        assert_relative_eq!(speed, 7.0, epsilon = EPSILON);
    }

    #[test]
    fn test_acceleration_limit() {
        let mut controller = LinearAccelerationBaseController::new(10.0, -15.0, 0.0);
        let t1 = Instant::now();
        controller.update(0.0, t1); // Initialize last_t

        let dt = 0.1;
        let t2 = future_instant(t1, dt);

        let max_possible_change = 10.0 * dt;

        // Target is far away, should be limited by max_acceleration
        let actual_speed = controller.update(100.0, t2);

        assert_relative_eq!(actual_speed, max_possible_change, epsilon = EPSILON);
    }

    #[test]
    fn test_deceleration_limit() {
        let mut controller = LinearAccelerationBaseController::new(10.0, -15.0, 20.0);
        let t1 = Instant::now();
        controller.update(20.0, t1); // Initialize last_t

        let dt = 0.1;
        let t2 = future_instant(t1, dt);

        let max_possible_change = -15.0 * dt;

        // Target is far away, should be limited by max_deceleration
        let actual_speed = controller.update(0.0, t2);

        assert_relative_eq!(actual_speed, 20.0 + max_possible_change, epsilon = EPSILON);
    }

    #[test]
    fn test_zero_time_delta() {
        let mut controller = LinearAccelerationBaseController::new(10.0, -15.0, 5.0);
        let now = Instant::now();

        controller.update(5.0, now); // Initialize last_t
        // Update again with the same time stamp
        let speed = controller.update(10.0, now);

        // With zero time delta, speed shouldn't change
        assert_relative_eq!(speed, 5.0, epsilon = EPSILON);
    }

    #[test]
    fn test_acceleration_capped_at_target() {
        let mut controller = LinearAccelerationBaseController::new(100.0, -15.0, 3.0);
        let t1 = Instant::now();
        controller.update(3.0, t1); // Initialize last_t

        // With a high acceleration of 100.0 and dt of 0.1,
        // we could go up by 10.0 units, from 3.0 to 13.0
        // But target is only 5.0, so we should cap at 5.0
        let dt = 0.1;
        let t2 = future_instant(t1, dt);

        let speed = controller.update(5.0, t2);

        // Speed should be capped at target_speed (5.0), not 13.0
        assert_relative_eq!(speed, 5.0, epsilon = EPSILON);
    }

    #[test]
    fn test_deceleration_capped_at_target() {
        let mut controller = LinearAccelerationBaseController::new(10.0, -100.0, 8.0);
        let t1 = Instant::now();
        controller.update(8.0, t1); // Initialize last_t

        // With a high deceleration of -100.0 and dt of 0.1,
        // we could go down by 10.0 units, from 8.0 to -2.0
        // But target is 5.0, so we should cap at 5.0
        let dt = 0.1;
        let t2 = future_instant(t1, dt);

        let speed = controller.update(5.0, t2);

        // Speed should be capped at target_speed (5.0), not -2.0
        assert_relative_eq!(speed, 5.0, epsilon = EPSILON);
    }

    #[test]
    fn test_exact_acceleration_to_target() {
        let mut controller = LinearAccelerationBaseController::new(20.0, -15.0, 3.0);
        let t1 = Instant::now();
        controller.update(3.0, t1); // Initialize last_t

        // With acceleration of 20.0 and dt of 0.1,
        // we would go up exactly 2.0 units, from 3.0 to 5.0
        // Target is 5.0, so we should hit it exactly
        let dt = 0.1;
        let t2 = future_instant(t1, dt);

        let speed = controller.update(5.0, t2);

        assert_relative_eq!(speed, 5.0, epsilon = EPSILON);
    }

    #[test]
    fn test_exact_deceleration_to_target() {
        let mut controller = LinearAccelerationBaseController::new(10.0, -30.0, 8.0);
        let t1 = Instant::now();
        controller.update(8.0, t1); // Initialize last_t

        // With deceleration of -30.0 and dt of 0.1,
        // we would go down exactly 3.0 units, from 8.0 to 5.0
        // Target is 5.0, so we should hit it exactly
        let dt = 0.1;
        let t2 = future_instant(t1, dt);

        let speed = controller.update(5.0, t2);

        assert_relative_eq!(speed, 5.0, epsilon = EPSILON);
    }
}
