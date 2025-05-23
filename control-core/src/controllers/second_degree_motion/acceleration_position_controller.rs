/// Controller for position-based motion with trapezoidal speed profile and acceleration limits.
///
/// This controller manages:
/// - Position (the primary controlled variable)
/// - Speed (the first derivative of position)
/// - Acceleration (the second derivative of position, rate of change of speed)
#[derive(Debug)]
pub struct AccelerationPositionController {
    // Configuration parameters
    max_speed: f64,        // Positive value for increasing position
    min_speed: f64,        // Negative value for decreasing position
    max_acceleration: f64, // Positive value for increasing speed
    min_acceleration: f64, // Negative value for decreasing speed

    // State variables
    current_position: f64,
    current_speed: f64,
    current_acceleration: f64,
    target_position: f64,

    // Internal state for motion planning
    motion_phase: MotionPhase,
    direction: i8, // Direction of position change: 1 (increase), -1 (decrease), 0 (none)
    peak_speed: f64, // Calculated peak speed for this motion
    deceleration_position: f64, // Position at which to start decreasing speed

    // Pre-calculated constants
    max_speed_change_pos: f64,
    max_speed_change_neg: f64,
    max_decel_change_pos: f64,
    max_decel_change_neg: f64,
}

/// Represents the current phase of motion
#[derive(Debug, Clone, Copy, PartialEq)]
enum MotionPhase {
    Idle,
    IncreasingSpeed,
    ConstantSpeed,
    DecreasingSpeed,
}

impl AccelerationPositionController {
    /// Create a new position controller with the given constraints
    ///
    /// Parameters:
    /// - max_speed: Maximum speed (positive value)
    /// - min_speed: Minimum speed (negative value)
    /// - max_acceleration: Maximum acceleration for increasing speed (positive value)
    /// - min_acceleration: Minimum acceleration for decreasing speed (negative value)
    pub fn new(
        max_speed: f64,
        min_speed: f64,
        max_acceleration: f64,
        min_acceleration: f64,
    ) -> Self {
        // Ensure min_speed and min_acceleration are negative
        let min_speed = min_speed.min(0.0);
        let min_acceleration = min_acceleration.min(0.0);

        // Ensure max_speed and max_acceleration are positive
        let max_speed = max_speed.max(0.0);
        let max_acceleration = max_acceleration.max(0.0);

        // Pre-calculate constants
        let max_speed_change_pos = max_speed.powi(2) / (2.0 * max_acceleration);
        let max_speed_change_neg = min_speed.powi(2) / (2.0 * max_acceleration);
        let max_decel_change_pos = max_speed.powi(2) / (2.0 * min_acceleration.abs());
        let max_decel_change_neg = min_speed.powi(2) / (2.0 * min_acceleration.abs());

        AccelerationPositionController {
            max_speed,
            min_speed,
            max_acceleration,
            min_acceleration,
            current_position: 0.0,
            current_speed: 0.0,
            current_acceleration: 0.0,
            target_position: 0.0,
            motion_phase: MotionPhase::Idle,
            direction: 0,
            peak_speed: 0.0,
            deceleration_position: 0.0,
            max_speed_change_pos,
            max_speed_change_neg,
            max_decel_change_pos,
            max_decel_change_neg,
        }
    }

    /// Update the controller state based on the target position and time step.
    ///
    /// Returns the updated position.
    pub fn update(&mut self, target_position: f64, dt: f64) -> f64 {
        // Check if target has changed
        if (target_position - self.target_position).abs() > 1e-6 {
            self.target_position = target_position;
            self.plan_motion();
        }

        // Update position based on current motion phase
        self.update_motion(dt);

        self.current_position
    }

    /// Plan the motion trajectory based on current state and target position.
    fn plan_motion(&mut self) {
        // Calculate position change needed
        let position_change = self.target_position - self.current_position;

        // Early exit if already at target
        if position_change.abs() < 1e-6 {
            self.direction = 0;
            self.motion_phase = MotionPhase::Idle;
            return;
        }

        // Determine direction of position change and select appropriate constants
        let (
            direction,
            cruise_speed,
            _max_speed_change,
            max_decel_change,
            acceleration_rate,
            deacceleration_rate,
        ) = if position_change > 0.0 {
            (
                1,
                self.max_speed,
                self.max_speed_change_pos,
                self.max_decel_change_pos,
                self.max_acceleration,
                self.min_acceleration.abs(),
            ) // Use absolute value of min_acceleration for calculations
        } else {
            (
                -1,
                self.min_speed,
                self.max_speed_change_neg,
                self.max_decel_change_neg,
                self.max_acceleration,
                self.min_acceleration.abs(),
            ) // Use absolute value of min_acceleration for calculations
        };

        self.direction = direction;

        // Absolute position change
        let abs_position_change = position_change.abs();

        // Calculate stopping position change from current speed
        let current_speed_in_direction = self.current_speed * self.direction as f64;
        if current_speed_in_direction > 0.0 {
            // If moving in target direction, calculate stopping position change
            let stopping_position_change =
                current_speed_in_direction.powi(2) / (2.0 * deacceleration_rate);

            // Check if we need to decrease speed immediately
            if abs_position_change <= stopping_position_change {
                self.motion_phase = MotionPhase::DecreasingSpeed;
                return;
            }
        }

        // Calculate speed change from current to cruise speed
        let speed_change_position = if current_speed_in_direction < cruise_speed.abs() {
            // Need to increase speed
            (cruise_speed.abs().powi(2) - current_speed_in_direction.powi(2))
                / (2.0 * acceleration_rate)
        } else {
            // Already at or above cruise speed
            0.0
        };

        // Calculate if we can reach cruise speed
        if abs_position_change < speed_change_position + max_decel_change {
            // Can't reach cruise speed - triangular profile
            // Calculate peak speed
            let accel_deaccel_ratio = acceleration_rate / deacceleration_rate;
            let term1 = 2.0 * abs_position_change * acceleration_rate * deacceleration_rate
                / (acceleration_rate + deacceleration_rate);
            let term2 = current_speed_in_direction.powi(2) * accel_deaccel_ratio;

            // Avoid square root when possible
            self.peak_speed = if term1 + term2 > 0.0 {
                (term1 + term2).sqrt() * cruise_speed.signum()
            } else {
                0.0
            };
        } else {
            // Can reach cruise speed - trapezoidal profile
            self.peak_speed = cruise_speed;
        }

        // Calculate deceleration position point
        let decel_position_change = self.peak_speed.abs().powi(2) / (2.0 * deacceleration_rate);

        // Calculate the position at which to start decreasing speed
        self.deceleration_position = if self.direction > 0 {
            self.target_position - decel_position_change
        } else {
            self.target_position + decel_position_change
        };

        // Start in speed increase phase if not already at peak speed
        if self.current_speed.abs() < self.peak_speed.abs() {
            self.motion_phase = MotionPhase::IncreasingSpeed;
        } else {
            // Already at or above peak speed
            self.motion_phase = MotionPhase::ConstantSpeed;
        }
    }

    /// Update position, speed, and acceleration based on current motion phase.
    fn update_motion(&mut self, dt: f64) {
        // Quick exit for idle state
        if self.motion_phase == MotionPhase::Idle {
            self.current_acceleration = 0.0;
            self.current_speed = 0.0;
            return;
        }

        // Update based on current motion phase
        match self.motion_phase {
            MotionPhase::IncreasingSpeed => {
                // Apply acceleration to increase speed
                self.current_acceleration = self.max_acceleration * self.direction as f64;

                // Update speed
                let mut new_speed = self.current_speed + self.current_acceleration * dt;

                // Check if we've reached peak speed
                if (self.direction > 0 && new_speed >= self.peak_speed)
                    || (self.direction < 0 && new_speed <= self.peak_speed)
                {
                    new_speed = self.peak_speed;
                    self.motion_phase = MotionPhase::ConstantSpeed;
                }

                self.current_speed = new_speed;
            }

            MotionPhase::ConstantSpeed => {
                // Maintain constant speed
                self.current_acceleration = 0.0;
                self.current_speed = self.peak_speed;

                // Check if we've reached the deceleration position point
                if (self.direction > 0 && self.current_position >= self.deceleration_position)
                    || (self.direction < 0 && self.current_position <= self.deceleration_position)
                {
                    self.motion_phase = MotionPhase::DecreasingSpeed;
                }
            }

            MotionPhase::DecreasingSpeed => {
                // Apply min_acceleration to decrease speed
                // Use min_acceleration (negative value) directly
                self.current_acceleration = self.min_acceleration * self.direction as f64;

                // Update speed
                let new_speed = self.current_speed + self.current_acceleration * dt;

                // Check if we've reached zero speed or changed direction
                if self.current_speed * new_speed <= 0.0 {
                    // Sign change or zero
                    self.current_speed = 0.0;
                    self.motion_phase = MotionPhase::Idle;
                    // Snap to target when stopping speed
                    self.current_position = self.target_position;
                    return;
                }

                self.current_speed = new_speed;
            }

            MotionPhase::Idle => {
                // This case is handled by the early return above
            }
        }

        // Update position based on current speed
        self.current_position += self.current_speed * dt;

        // Check if we've reached the target with near-zero speed
        if (self.current_position - self.target_position).abs() < 1e-6
            && self.current_speed.abs() < 1e-6
        {
            self.current_position = self.target_position; // Snap to exact target
            self.current_speed = 0.0;
            self.current_acceleration = 0.0;
            self.motion_phase = MotionPhase::Idle;
        }
    }

    /// Get the current position
    pub fn get_position(&self) -> f64 {
        self.current_position
    }

    /// Get the current speed
    pub fn get_speed(&self) -> f64 {
        self.current_speed
    }

    /// Get the current acceleration
    pub fn get_acceleration(&self) -> f64 {
        self.current_acceleration
    }

    /// Get the target position
    pub fn get_target_position(&self) -> f64 {
        self.target_position
    }

    /// Set the maximum speed
    pub fn set_max_speed(&mut self, max_speed: f64) {
        self.max_speed = max_speed;
        self.plan_motion(); // Recalculate motion plan
    }

    /// Set the minimum speed
    pub fn set_min_speed(&mut self, min_speed: f64) {
        self.min_speed = min_speed;
        self.plan_motion(); // Recalculate motion plan
    }

    /// Set the maximum acceleration
    pub fn set_max_acceleration(&mut self, max_acceleration: f64) {
        self.max_acceleration = max_acceleration;
        self.plan_motion(); // Recalculate motion plan
    }

    /// Set the minimum acceleration
    pub fn set_min_acceleration(&mut self, min_acceleration: f64) {
        self.min_acceleration = min_acceleration;
        self.plan_motion(); // Recalculate motion plan
    }
}
