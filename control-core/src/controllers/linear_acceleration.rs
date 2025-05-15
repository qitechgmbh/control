use std::time::Instant;

pub struct LinearAccelerationController {
    /// Maximum acceleration in units per second (positive value)
    pub max_acceleration: f64,

    /// Maximum deceleration in units per second (negative value)
    pub max_deceleration: f64,

    /// Calculated speed at the last update
    speed: f64,

    /// Last update time
    last_t: Option<Instant>,
}

impl LinearAccelerationController {
    pub fn new(max_acceleration: f64, max_deceleration: f64, initial_speed: f64) -> Self {
        // Ensure max_deceleration is negative
        let max_deceleration = if max_deceleration > 0.0 {
            -max_deceleration
        } else {
            max_deceleration
        };

        Self {
            max_acceleration,
            max_deceleration,
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
            let speed_change = self.max_acceleration * dt;
            let new_speed = (self.speed + speed_change).min(target_speed);
            self.speed = new_speed;
            return new_speed;
        }

        // Need to decelerate (max_deceleration is negative)
        if self.speed > target_speed {
            let speed_change = self.max_deceleration * dt; // This will be negative
            let new_speed = (self.speed + speed_change).max(target_speed);
            self.speed = new_speed;
            return new_speed;
        }

        self.speed
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
        let controller = LinearAccelerationController::new(10.0, -15.0, 5.0);
        assert_relative_eq!(controller.max_acceleration, 10.0, epsilon = EPSILON);
        assert_relative_eq!(controller.max_deceleration, -15.0, epsilon = EPSILON);
        assert_relative_eq!(controller.speed, 5.0, epsilon = EPSILON);
        assert!(controller.last_t.is_none());
    }

    #[test]
    fn test_initialization_positive_deceleration() {
        // Test that positive deceleration values are converted to negative
        let controller = LinearAccelerationController::new(10.0, 15.0, 5.0);
        assert_relative_eq!(controller.max_deceleration, -15.0, epsilon = EPSILON);
    }

    #[test]
    fn test_first_update() {
        let mut controller = LinearAccelerationController::new(10.0, -15.0, 5.0);
        let now = Instant::now();

        let speed = controller.update(5.0, now);

        assert_relative_eq!(speed, 5.0, epsilon = EPSILON);
        assert_relative_eq!(controller.speed, 5.0, epsilon = EPSILON);
        assert_eq!(controller.last_t.unwrap(), now);
    }

    #[test]
    fn test_acceleration() {
        let mut controller = LinearAccelerationController::new(10.0, -15.0, 0.0);
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
        let mut controller = LinearAccelerationController::new(10.0, -15.0, 10.0);
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
        let mut controller = LinearAccelerationController::new(10.0, -15.0, 7.0);
        let t1 = Instant::now();
        controller.update(7.0, t1); // Initialize last_t

        let t2 = future_instant(t1, 0.1);

        let speed = controller.update(7.0, t2);

        assert_relative_eq!(speed, 7.0, epsilon = EPSILON);
    }

    #[test]
    fn test_acceleration_limit() {
        let mut controller = LinearAccelerationController::new(10.0, -15.0, 0.0);
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
        let mut controller = LinearAccelerationController::new(10.0, -15.0, 20.0);
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
        let mut controller = LinearAccelerationController::new(10.0, -15.0, 5.0);
        let now = Instant::now();

        controller.update(5.0, now); // Initialize last_t
        // Update again with the same time stamp
        let speed = controller.update(10.0, now);

        // With zero time delta, speed shouldn't change
        assert_relative_eq!(speed, 5.0, epsilon = EPSILON);
    }

    #[test]
    fn test_acceleration_capped_at_target() {
        let mut controller = LinearAccelerationController::new(100.0, -15.0, 3.0);
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
        let mut controller = LinearAccelerationController::new(10.0, -100.0, 8.0);
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
        let mut controller = LinearAccelerationController::new(20.0, -15.0, 3.0);
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
        let mut controller = LinearAccelerationController::new(10.0, -30.0, 8.0);
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
