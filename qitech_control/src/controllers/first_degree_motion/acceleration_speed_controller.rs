use std::time::Instant;

#[derive(Debug)]
pub struct AccelerationSpeedController {
    /// Maximum deceleration in units per second (positive value)
    max_acceleration: f64,

    /// Maximum acceleratoin in units per second (negative value)
    min_acceleration: f64,

    /// Minimum speed limit (None for no limit)
    min_speed: Option<f64>,

    /// Maximum speed limit (None for no limit)
    max_speed: Option<f64>,

    /// Calculated speed at the last update
    last_speed: f64,

    /// Last update time
    last_t: Option<Instant>,
}

impl AccelerationSpeedController {
    pub const fn new(
        min_speed: Option<f64>,
        max_speed: Option<f64>,
        min_acceleration: f64,
        max_acceleration: f64,
        initial_speed: f64,
    ) -> Self {
        Self {
            min_acceleration,
            max_acceleration,
            min_speed,
            max_speed,
            last_speed: initial_speed,
            last_t: None,
        }
    }

    /// Creates a new acceleration speed controller with simplified parameters.
    /// Sets min_acceleration to -max_acceleration for symmetric behavior.
    /// No speed limits are applied.
    pub fn new_simple(max_acceleration: f64, initial_speed: f64) -> Self {
        Self::new(
            None,              // min_speed
            None,              // max_speed
            -max_acceleration, // min_acceleration (deceleration)
            max_acceleration,  // max_acceleration
            initial_speed,
        )
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

        // Get acceleration
        let acceleration = if target_speed > self.last_speed {
            // We are accelerating
            self.max_acceleration
        } else if target_speed < self.last_speed {
            // We are decelerating
            self.min_acceleration
        } else {
            0.0
        };

        let speed_change = acceleration * dt;
        let new_speed = self.last_speed + speed_change;

        // Prevent overshooting the target speed
        let new_speed = if acceleration > 0.0 {
            // Limit speed when accelerating
            new_speed.min(target_speed)
        } else if acceleration < 0.0 {
            // Limit speed when decelerating
            new_speed.max(target_speed)
        } else {
            new_speed
        };

        // Apply speed limits
        let new_speed = self.apply_speed_limits(new_speed);

        self.last_speed = new_speed;

        new_speed
    }

    pub const fn reset(&mut self, initial_speed: f64) {
        self.last_speed = initial_speed;
        self.last_t = None; // Reset the last update time
    }

    pub const fn set_max_acceleration(&mut self, acceleration: f64) {
        self.max_acceleration = acceleration;
    }

    const fn apply_speed_limits(&self, speed: f64) -> f64 {
        let mut limited_speed = speed;

        if let Some(min) = self.min_speed {
            limited_speed = limited_speed.max(min);
        }

        if let Some(max) = self.max_speed {
            limited_speed = limited_speed.min(max);
        }

        limited_speed
    }

    pub const fn get_min_speed(&self) -> Option<f64> {
        self.min_speed
    }

    pub const fn get_max_speed(&self) -> Option<f64> {
        self.max_speed
    }

    pub const fn set_min_speed(&mut self, min_speed: Option<f64>) {
        self.min_speed = min_speed;
    }

    pub const fn set_max_speed(&mut self, max_speed: Option<f64>) {
        self.max_speed = max_speed;
    }

    pub const fn set_min_acceleration(&mut self, deceleration: f64) {
        self.min_acceleration = deceleration;
    }
}
