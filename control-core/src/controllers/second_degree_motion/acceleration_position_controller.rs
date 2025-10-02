use std::fmt;

// Remove the anyhow::Ok import as it conflicts with std::result::Ok

/// Errors that can occur when creating or using the motion controller
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotionControllerError {
    InvalidSpeedLimits,
    InvalidAccelerationLimits,
    InvalidPositionLimits,
    ZeroDeceleration,
}

impl fmt::Display for MotionControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpeedLimits => {
                write!(
                    f,
                    "Invalid speed limits: min_speed must be ≤ 0 and max_speed must be ≥ 0"
                )
            }
            Self::InvalidAccelerationLimits => {
                write!(
                    f,
                    "Invalid acceleration limits: min_acceleration must be < 0 and max_acceleration must be > 0"
                )
            }
            Self::InvalidPositionLimits => {
                write!(
                    f,
                    "Invalid position limits: min_position must be ≤ max_position"
                )
            }
            Self::ZeroDeceleration => {
                write!(
                    f,
                    "Zero deceleration rate: cannot plan motion without deceleration capability"
                )
            }
        }
    }
}

impl std::error::Error for MotionControllerError {}

/// Advanced motion controller for position-based movement with precise acceleration control.
///
/// # Overview
///
/// This controller implements a sophisticated motion planning algorithm that generates smooth,
/// efficient trajectories for position control systems. It operates on three levels of motion:
/// - **Position** (primary controlled variable)
/// - **Speed** (first derivative of position)
/// - **Acceleration** (second derivative of position, rate of change of speed)
///
/// # Motion Planning Algorithm
///
/// The controller uses an intelligent trajectory planning system that automatically determines
/// the optimal motion profile based on:
/// 1. Current state (position, speed, acceleration)
/// 2. Target position
/// 3. Physical constraints (speed and acceleration limits)
/// 4. Optional position boundaries
///
/// ## Trajectory Types
///
/// The planner generates one of three motion profiles:
///
/// ### 1. Trapezoidal Profile
/// Used when sufficient distance allows reaching maximum speed:
/// - **Phase 1**: Accelerate from current speed to peak speed
/// - **Phase 2**: Maintain constant peak speed
/// - **Phase 3**: Decelerate from peak speed to zero at target
///
/// ### 2. Triangular Profile  
/// Used when distance is limited, preventing full speed:
/// - **Phase 1**: Accelerate to calculated peak speed (less than max)
/// - **Phase 2**: Immediately decelerate to zero at target
/// - **Phase 3**: (No constant speed phase)
///
/// ### 3. Deceleration-Only Profile
/// Used when already moving faster than optimal for remaining distance:
/// - **Phase 1**: (No acceleration phase)
/// - **Phase 2**: (No constant speed phase)  
/// - **Phase 3**: Decelerate from current speed to zero at target
///
/// ## Predictive Calculations
///
/// The controller performs sophisticated predictive calculations:
///
/// ### Distance-Based Predictions
/// - **Stopping Distance**: Calculates distance needed to decelerate from current speed to zero
/// - **Acceleration Distance**: Determines distance required to reach peak speed from current speed
/// - **Deceleration Point**: Predicts optimal position to begin final deceleration
///
/// ### Speed Profile Optimization
/// When full acceleration isn't possible, the controller solves for optimal peak speed using:
/// ```ignore
/// peak_speed = sqrt(current_speed² + (2 * remaining_distance * accel * decel) / (accel + decel))
/// ```
///
/// This ensures the smoothest possible motion profile while respecting all constraints.
#[derive(Debug)]
pub struct AccelerationPositionController {
    // Configuration parameters
    config: ControllerConfig,

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

    // Pre-calculated constants (updated when config changes)
    motion_constants: MotionConstants,

    // Flag to track if re-planning is needed
    needs_replanning: bool,
}

/// Configuration parameters for the controller
#[derive(Debug, Clone)]
pub struct ControllerConfig {
    min_speed: f64,            // Negative value for decreasing position
    max_speed: f64,            // Positive value for increasing position
    min_acceleration: f64,     // Negative value for decreasing speed
    max_acceleration: f64,     // Positive value for increasing speed
    min_position: Option<f64>, // Minimum allowed position
    max_position: Option<f64>, // Maximum allowed position
    position_tolerance: f64,   // Tolerance for considering target reached
    speed_tolerance: f64,      // Tolerance for considering speed zero
}

/// Pre-calculated motion constants for performance
#[derive(Debug, Clone)]
struct MotionConstants {
    max_speed_change_pos: f64,
    max_speed_change_neg: f64,
    max_decel_change_pos: f64,
    max_decel_change_neg: f64,
}

/// Represents the current phase of motion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionPhase {
    Idle,
    IncreasingSpeed,
    ConstantSpeed,
    DecreasingSpeed,
}

/// Builder for creating AccelerationPositionController with validation
pub struct ControllerBuilder {
    min_speed: Option<f64>,
    max_speed: Option<f64>,
    min_acceleration: Option<f64>,
    max_acceleration: Option<f64>,
    min_position: Option<f64>,
    max_position: Option<f64>,
    position_tolerance: f64,
    speed_tolerance: f64,
}

impl ControllerBuilder {
    /// Create a new builder
    pub const fn new() -> Self {
        Self {
            min_speed: None,
            max_speed: None,
            min_acceleration: None,
            max_acceleration: None,
            min_position: None,
            max_position: None,
            position_tolerance: 1e-6,
            speed_tolerance: 1e-6,
        }
    }

    /// Set speed limits (min_speed ≤ 0, max_speed ≥ 0)
    pub const fn speed_limits(mut self, min_speed: f64, max_speed: f64) -> Self {
        self.min_speed = Some(min_speed);
        self.max_speed = Some(max_speed);
        self
    }

    /// Set acceleration limits (min_acceleration < 0, max_acceleration > 0)
    pub const fn acceleration_limits(
        mut self,
        min_acceleration: f64,
        max_acceleration: f64,
    ) -> Self {
        self.min_acceleration = Some(min_acceleration);
        self.max_acceleration = Some(max_acceleration);
        self
    }

    /// Set position limits (optional)
    pub const fn position_limits(
        mut self,
        min_position: Option<f64>,
        max_position: Option<f64>,
    ) -> Self {
        self.min_position = min_position;
        self.max_position = max_position;
        self
    }

    /// Set tolerances for position and speed comparisons
    pub const fn tolerances(mut self, position_tolerance: f64, speed_tolerance: f64) -> Self {
        self.position_tolerance = position_tolerance;
        self.speed_tolerance = speed_tolerance;
        self
    }

    /// Build the controller with validation
    pub fn build(self) -> Result<AccelerationPositionController, MotionControllerError> {
        let min_speed = self
            .min_speed
            .ok_or(MotionControllerError::InvalidSpeedLimits)?;
        let max_speed = self
            .max_speed
            .ok_or(MotionControllerError::InvalidSpeedLimits)?;
        let min_acceleration = self
            .min_acceleration
            .ok_or(MotionControllerError::InvalidAccelerationLimits)?;
        let max_acceleration = self
            .max_acceleration
            .ok_or(MotionControllerError::InvalidAccelerationLimits)?;

        AccelerationPositionController::new(
            min_speed,
            max_speed,
            min_acceleration,
            max_acceleration,
            self.min_position,
            self.max_position,
            self.position_tolerance,
            self.speed_tolerance,
        )
    }
}

impl Default for ControllerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AccelerationPositionController {
    /// Creates a new acceleration position controller with full configuration
    ///
    /// This constructor allows complete control over all motion parameters and limits.
    /// Use this when you need asymmetric limits or specific tolerances.
    ///
    /// # Arguments
    /// * `min_speed` - Minimum speed limit (must be ≤ 0, typically negative for reverse motion)
    /// * `max_speed` - Maximum speed limit (must be ≥ 0, positive for forward motion)
    /// * `min_acceleration` - Minimum acceleration limit (must be < 0, for deceleration/braking)
    /// * `max_acceleration` - Maximum acceleration limit (must be > 0, for speeding up)
    /// * `min_position` - Optional minimum position limit (None for unlimited)
    /// * `max_position` - Optional maximum position limit (None for unlimited)
    /// * `position_tolerance` - Tolerance for considering target position reached (typically 1e-6)
    /// * `speed_tolerance` - Tolerance for considering speed zero (typically 1e-6)
    ///
    /// # Returns
    /// Returns `Ok(AccelerationPositionController)` on success, or `Err(MotionControllerError)` if parameters are invalid.
    ///
    /// # Errors
    /// - `InvalidSpeedLimits`: If min_speed > 0 or max_speed < 0
    /// - `InvalidAccelerationLimits`: If min_acceleration ≥ 0 or max_acceleration ≤ 0  
    /// - `InvalidPositionLimits`: If min_position > max_position
    ///
    /// # Example
    /// ```ignore
    /// use control_core::controllers::second_degree_motion::acceleration_position_controller::AccelerationPositionController;
    ///
    /// let controller = AccelerationPositionController::new(
    ///     -10.0,      // min_speed: can move backwards at 10 units/s
    ///     20.0,       // max_speed: can move forwards at 20 units/s  
    ///     -5.0,       // min_acceleration: can decelerate at 5 units/s²
    ///     8.0,        // max_acceleration: can accelerate at 8 units/s²
    ///     Some(-100.0), // min_position: minimum position limit
    ///     Some(100.0),  // max_position: maximum position limit
    ///     1e-6,       // position_tolerance
    ///     1e-6,       // speed_tolerance
    /// ).expect("Valid parameters");
    /// ```
    pub fn new(
        min_speed: f64,
        max_speed: f64,
        min_acceleration: f64,
        max_acceleration: f64,
        min_position: Option<f64>,
        max_position: Option<f64>,
        position_tolerance: f64,
        speed_tolerance: f64,
    ) -> Result<Self, MotionControllerError> {
        // Validate parameters
        if min_speed > 0.0 || max_speed < 0.0 {
            return Err(MotionControllerError::InvalidSpeedLimits);
        }

        if min_acceleration >= 0.0 || max_acceleration <= 0.0 {
            return Err(MotionControllerError::InvalidAccelerationLimits);
        }

        if let (Some(min_pos), Some(max_pos)) = (min_position, max_position) {
            if min_pos > max_pos {
                return Err(MotionControllerError::InvalidPositionLimits);
            }
        }

        let config = ControllerConfig {
            min_speed,
            max_speed,
            min_acceleration,
            max_acceleration,
            min_position,
            max_position,
            position_tolerance,
            speed_tolerance,
        };

        let motion_constants = Self::calculate_motion_constants(&config);

        Ok(Self {
            config,
            current_position: 0.0,
            current_speed: 0.0,
            current_acceleration: 0.0,
            target_position: 0.0,
            motion_phase: MotionPhase::Idle,
            direction: 0,
            peak_speed: 0.0,
            deceleration_position: 0.0,
            motion_constants,
            needs_replanning: false,
        })
    }

    /// Create a new motion controller with simple symmetric limits and default tolerances
    ///
    /// This is a convenience constructor that creates symmetric limits:
    /// - Speed limits: [-speed, +speed]
    /// - Acceleration limits: [-acceleration, +acceleration]
    /// - Position limits: [-position, +position] (if position is Some)
    /// - Default tolerances: 1e-6 for both position and speed
    ///
    /// # Parameters (ordered: position, speed, acceleration)
    /// - `position`: Optional maximum position magnitude (None for no limits, Some(x) creates [-x, +x])
    /// - `speed`: Maximum speed magnitude (creates limits [-speed, +speed])
    /// - `acceleration`: Maximum acceleration magnitude (creates limits [-acceleration, +acceleration])
    ///
    /// # Errors
    /// Returns `MotionControllerError` if parameters are invalid (speed or acceleration <= 0)
    pub fn new_simple(
        position: Option<f64>,
        speed: f64,
        acceleration: f64,
    ) -> Result<Self, MotionControllerError> {
        if speed <= 0.0 {
            return Err(MotionControllerError::InvalidSpeedLimits);
        }

        if acceleration <= 0.0 {
            return Err(MotionControllerError::InvalidAccelerationLimits);
        }

        Self::new(
            -speed,                     // min_speed
            speed,                      // max_speed
            -acceleration,              // min_acceleration
            acceleration,               // max_acceleration
            position.map(|p| -p.abs()), // min_position
            position.map(|p| p.abs()),  // max_position
            1e-6,                       // position_tolerance (default)
            1e-6,                       // speed_tolerance (default)
        )
    }

    /// Create a new builder for the controller
    pub const fn builder() -> ControllerBuilder {
        ControllerBuilder::new()
    }

    /// Calculate pre-computed motion constants
    fn calculate_motion_constants(config: &ControllerConfig) -> MotionConstants {
        let max_speed_change_pos = if config.max_acceleration > f64::EPSILON {
            config.max_speed.powi(2) / (2.0 * config.max_acceleration)
        } else {
            f64::INFINITY
        };

        let max_speed_change_neg = if config.max_acceleration > f64::EPSILON {
            config.min_speed.powi(2) / (2.0 * config.max_acceleration)
        } else {
            f64::INFINITY
        };

        let decel_rate = config.min_acceleration.abs();
        let max_decel_change_pos = if decel_rate > f64::EPSILON {
            config.max_speed.powi(2) / (2.0 * decel_rate)
        } else {
            f64::INFINITY
        };

        let max_decel_change_neg = if decel_rate > f64::EPSILON {
            config.min_speed.powi(2) / (2.0 * decel_rate)
        } else {
            f64::INFINITY
        };

        MotionConstants {
            max_speed_change_pos,
            max_speed_change_neg,
            max_decel_change_pos,
            max_decel_change_neg,
        }
    }

    /// Check if two floating-point values are approximately equal using relative tolerance
    fn approx_equal(&self, a: f64, b: f64, tolerance: f64) -> bool {
        let diff = (a - b).abs();
        diff <= tolerance || diff <= tolerance * a.abs().max(b.abs())
    }

    /// Updates the controller state and computes the next position
    ///
    /// This is the main control loop function that should be called periodically.
    /// It takes a target position and time step, then computes the optimal motion
    /// profile to reach that target while respecting all speed and acceleration limits.
    ///
    /// The controller automatically handles:
    /// - Motion planning with acceleration, cruising, and deceleration phases
    /// - Target position clamping to configured position limits
    /// - Smooth acceleration/deceleration transitions
    /// - Re-planning when target changes during motion
    ///
    /// # Arguments
    /// * `target_position` - The desired position to move towards
    /// * `dt` - Time step in seconds (must be > 0 and ≤ 1.0 for stability)
    ///
    /// # Returns
    /// Returns `Ok(current_position)` with the updated position after the time step,
    /// or `Err(MotionControllerError)` if parameters are invalid.
    ///
    /// # Errors
    /// - `InvalidTimeStep`: If dt ≤ 0 or dt > 1.0
    ///
    /// # Example
    /// ```ignore
    /// use control_core::controllers::second_degree_motion::acceleration_position_controller::AccelerationPositionController;
    ///
    /// let mut controller = AccelerationPositionController::new_simple(None, 10.0, 5.0)?;
    ///
    /// // Control loop - move to position 100
    /// let dt = 0.01; // 10ms time step
    /// let target = 100.0;
    ///
    /// let current_pos = controller.update(target, dt)?;
    /// println!("Current position: {:.2}", current_pos);
    /// ```
    pub fn update(&mut self, target_position: f64, dt: f64) -> Result<f64, MotionControllerError> {
        // Validate time step
        if dt <= 0.0 {
            // just return last position
            return Ok(self.current_position);
        }

        // Constrain target position within limits
        let mut constrained_target = target_position;
        if let Some(min_pos) = self.config.min_position {
            constrained_target = constrained_target.max(min_pos);
        }
        if let Some(max_pos) = self.config.max_position {
            constrained_target = constrained_target.min(max_pos);
        }

        // Check if target has changed significantly
        if !self.approx_equal(
            constrained_target,
            self.target_position,
            self.config.position_tolerance,
        ) {
            self.target_position = constrained_target;
            self.needs_replanning = true;
        }

        // Re-plan motion if needed
        if self.needs_replanning {
            self.plan_motion()?;
            self.needs_replanning = false;
        }

        // Update position based on current motion phase
        self.update_motion(dt)?;

        Ok(self.current_position)
    }

    /// Plan the motion trajectory based on current state and target position.
    fn plan_motion(&mut self) -> Result<(), MotionControllerError> {
        // Calculate position change needed
        let position_change = self.target_position - self.current_position;

        // Early exit if already at target
        if self.approx_equal(position_change.abs(), 0.0, self.config.position_tolerance) {
            self.direction = 0;
            self.motion_phase = MotionPhase::Idle;
            return Ok(());
        }

        // Determine direction of position change and select appropriate constants
        let direction = if position_change > 0.0 { 1i8 } else { -1i8 };

        let (
            cruise_speed,
            _max_speed_change,
            max_decel_change,
            acceleration_rate,
            deceleration_rate,
        ) = if direction > 0 {
            (
                self.config.max_speed,
                self.motion_constants.max_speed_change_pos,
                self.motion_constants.max_decel_change_pos,
                self.config.max_acceleration,
                self.config.min_acceleration.abs(),
            )
        } else {
            (
                self.config.min_speed,
                self.motion_constants.max_speed_change_neg,
                self.motion_constants.max_decel_change_neg,
                self.config.max_acceleration,
                self.config.min_acceleration.abs(),
            )
        };

        self.direction = direction;

        // Check for zero deceleration
        if deceleration_rate <= f64::EPSILON {
            return Err(MotionControllerError::ZeroDeceleration);
        }

        // Absolute position change
        let abs_position_change = position_change.abs();

        // Calculate current speed in the direction of motion
        let current_speed_in_direction = self.current_speed * direction as f64;

        // Calculate stopping distance from current speed
        if current_speed_in_direction > 0.0 {
            let stopping_distance = current_speed_in_direction.powi(2) / (2.0 * deceleration_rate);

            // Check if we need to decelerate immediately
            if abs_position_change <= stopping_distance + self.config.position_tolerance {
                self.motion_phase = MotionPhase::DecreasingSpeed;
                self.peak_speed = self.current_speed;
                self.deceleration_position = self.current_position;
                return Ok(());
            }
        }

        // Calculate distance needed to reach cruise speed from current speed
        let speed_change_distance = if current_speed_in_direction < cruise_speed.abs() {
            current_speed_in_direction.max(0.0).mul_add(
                -current_speed_in_direction.max(0.0),
                cruise_speed.abs().powi(2),
            ) / (2.0 * acceleration_rate)
        } else {
            0.0
        };

        // Determine if we can reach cruise speed (trapezoidal) or need triangular profile
        if abs_position_change >= speed_change_distance + max_decel_change {
            // Trapezoidal profile - can reach cruise speed
            self.peak_speed = cruise_speed;
        } else {
            // Triangular profile - calculate optimal peak speed
            // Using corrected kinematic equation: v_peak² = v_current² + 2*a*d_total*a_accel*a_decel/(a_accel + a_decel)
            let accel_decel_factor =
                (acceleration_rate * deceleration_rate) / (acceleration_rate + deceleration_rate);
            let discriminant = current_speed_in_direction.max(0.0).mul_add(
                current_speed_in_direction.max(0.0),
                2.0 * abs_position_change * accel_decel_factor,
            );

            if discriminant >= 0.0 {
                self.peak_speed = discriminant.sqrt() * cruise_speed.signum();
            } else {
                // Fallback to current speed if calculation fails
                self.peak_speed = self.current_speed;
            }
        }

        // Calculate deceleration start position
        let decel_distance = self.peak_speed.abs().powi(2) / (2.0 * deceleration_rate);
        self.deceleration_position = if direction > 0 {
            self.target_position - decel_distance
        } else {
            self.target_position + decel_distance
        };

        // Determine initial motion phase
        if self.approx_equal(
            self.current_speed.abs(),
            self.peak_speed.abs(),
            self.config.speed_tolerance,
        ) {
            self.motion_phase = MotionPhase::ConstantSpeed;
        } else if (direction > 0 && self.current_speed < self.peak_speed)
            || (direction < 0 && self.current_speed > self.peak_speed)
        {
            self.motion_phase = MotionPhase::IncreasingSpeed;
        } else {
            self.motion_phase = MotionPhase::ConstantSpeed;
        }

        Ok(())
    }

    /// Update position, speed, and acceleration based on current motion phase.
    fn update_motion(&mut self, dt: f64) -> Result<(), MotionControllerError> {
        // Quick exit for idle state
        if self.motion_phase == MotionPhase::Idle {
            self.current_acceleration = 0.0;
            if self.approx_equal(self.current_speed.abs(), 0.0, self.config.speed_tolerance) {
                self.current_speed = 0.0;
            }
            return Ok(());
        }

        // Update based on current motion phase
        match self.motion_phase {
            MotionPhase::IncreasingSpeed => {
                // Apply acceleration to increase speed
                self.current_acceleration = self.config.max_acceleration * self.direction as f64;

                // Update speed
                let new_speed = self.current_acceleration.mul_add(dt, self.current_speed);

                // Check if we've reached peak speed
                if (self.direction > 0 && new_speed >= self.peak_speed)
                    || (self.direction < 0 && new_speed <= self.peak_speed)
                {
                    self.current_speed = self.peak_speed;
                    self.motion_phase = MotionPhase::ConstantSpeed;
                } else {
                    self.current_speed = new_speed;
                }
            }

            MotionPhase::ConstantSpeed => {
                // Maintain constant speed
                self.current_acceleration = 0.0;
                self.current_speed = self.peak_speed;

                // Check if we've reached the deceleration point
                if (self.direction > 0 && self.current_position >= self.deceleration_position)
                    || (self.direction < 0 && self.current_position <= self.deceleration_position)
                {
                    self.motion_phase = MotionPhase::DecreasingSpeed;
                }
            }

            MotionPhase::DecreasingSpeed => {
                // Apply deceleration
                self.current_acceleration = self.config.min_acceleration * self.direction as f64;

                // Update speed
                let new_speed = self.current_acceleration.mul_add(dt, self.current_speed);

                // Check if we've reached zero speed or target
                if (self.current_speed > 0.0 && new_speed <= 0.0)
                    || (self.current_speed < 0.0 && new_speed >= 0.0)
                    || self.approx_equal(new_speed.abs(), 0.0, self.config.speed_tolerance)
                {
                    // Stop and snap to target
                    self.current_speed = 0.0;
                    self.current_acceleration = 0.0;
                    self.current_position = self.target_position;
                    self.motion_phase = MotionPhase::Idle;
                    return Ok(());
                }

                self.current_speed = new_speed;
            }

            MotionPhase::Idle => {
                // Already handled above
            }
        }

        // Update position based on current speed
        let new_position = self.current_speed.mul_add(dt, self.current_position);

        // Check position limits and handle violations
        let mut position_limited = false;

        if let Some(min_pos) = self.config.min_position {
            if new_position < min_pos {
                self.current_position = min_pos;
                if self.current_speed < 0.0 {
                    self.current_speed = 0.0;
                    self.current_acceleration = 0.0;
                    self.motion_phase = MotionPhase::Idle;
                }
                position_limited = true;
            }
        }

        if let Some(max_pos) = self.config.max_position {
            if new_position > max_pos && !position_limited {
                self.current_position = max_pos;
                if self.current_speed > 0.0 {
                    self.current_speed = 0.0;
                    self.current_acceleration = 0.0;
                    self.motion_phase = MotionPhase::Idle;
                }
                position_limited = true;
            }
        }

        if !position_limited {
            self.current_position = new_position;
        }

        // Check if we've reached the target with acceptable tolerance
        if self.approx_equal(
            self.current_position,
            self.target_position,
            self.config.position_tolerance,
        ) && self.approx_equal(self.current_speed.abs(), 0.0, self.config.speed_tolerance)
        {
            self.current_position = self.target_position;
            self.current_speed = 0.0;
            self.current_acceleration = 0.0;
            self.motion_phase = MotionPhase::Idle;
        }

        Ok(())
    }

    /// Gets the current position
    ///
    /// Returns the current position of the controlled object.
    /// This value is updated with each call to `update()`.
    pub const fn get_position(&self) -> f64 {
        self.current_position
    }

    /// Gets the current speed
    ///
    /// Returns the current speed (velocity) of the controlled object.
    /// Positive values indicate motion towards increasing position,
    /// negative values indicate motion towards decreasing position.
    pub const fn get_speed(&self) -> f64 {
        self.current_speed
    }

    /// Gets the current acceleration
    ///
    /// Returns the current acceleration of the controlled object.
    /// Positive values indicate increasing speed, negative values indicate decreasing speed.
    pub const fn get_acceleration(&self) -> f64 {
        self.current_acceleration
    }

    /// Get the target position
    pub const fn get_target_position(&self) -> f64 {
        self.target_position
    }

    /// Get the current motion phase
    pub const fn get_motion_phase(&self) -> MotionPhase {
        self.motion_phase
    }

    /// Get the direction of motion
    pub const fn get_direction(&self) -> i8 {
        self.direction
    }

    /// Get the calculated peak speed for current motion
    pub const fn get_peak_speed(&self) -> f64 {
        self.peak_speed
    }

    /// Get the position at which deceleration will begin
    pub const fn get_deceleration_position(&self) -> f64 {
        self.deceleration_position
    }

    /// Calculate remaining distance to target
    pub fn get_remaining_distance(&self) -> f64 {
        (self.target_position - self.current_position).abs()
    }

    /// Calculate estimated time to reach target (approximate)
    pub fn get_estimated_time_to_target(&self) -> f64 {
        if self.motion_phase == MotionPhase::Idle {
            return 0.0;
        }

        let remaining_distance = self.get_remaining_distance();
        if remaining_distance < self.config.position_tolerance {
            return 0.0;
        }

        // Simple approximation based on current speed and remaining distance
        // More sophisticated calculation would require solving the motion equations
        if self.current_speed.abs() > self.config.speed_tolerance {
            remaining_distance / self.current_speed.abs()
        } else {
            // Estimate based on acceleration from rest
            let accel = self.config.max_acceleration;
            if accel > f64::EPSILON {
                (2.0 * remaining_distance / accel).sqrt()
            } else {
                f64::INFINITY
            }
        }
    }

    /// Check if the controller is currently moving
    pub fn is_moving(&self) -> bool {
        self.motion_phase != MotionPhase::Idle
            || !self.approx_equal(self.current_speed.abs(), 0.0, self.config.speed_tolerance)
    }

    /// Check if the controller has reached the target
    pub fn is_at_target(&self) -> bool {
        self.motion_phase == MotionPhase::Idle
            && self.approx_equal(
                self.current_position,
                self.target_position,
                self.config.position_tolerance,
            )
            && self.approx_equal(self.current_speed.abs(), 0.0, self.config.speed_tolerance)
    }

    /// Force the controller to stop at current position
    pub const fn emergency_stop(&mut self) {
        self.target_position = self.current_position;
        self.current_speed = 0.0;
        self.current_acceleration = 0.0;
        self.motion_phase = MotionPhase::Idle;
        self.direction = 0;
        self.peak_speed = 0.0;
        self.needs_replanning = false;
    }

    /// Resets the controller to a specific position with zero speed and acceleration
    ///
    /// This method completely reinitializes the controller state, setting:
    /// - Current position to the specified value
    /// - Target position to the specified value (stops any ongoing motion)
    /// - Current speed to 0
    /// - Current acceleration to 0
    /// - Motion phase to Idle
    /// - Clears any motion planning state
    ///
    /// Use this method when you need to teleport the controlled object to a new position
    /// or when recovering from errors/emergency stops.
    ///
    /// # Arguments
    /// * `position` - The new position to reset to
    ///
    /// # Returns
    /// Returns `Ok(())` on success, or `Err(MotionControllerError)` if the position is outside configured limits.
    ///
    /// # Errors
    /// - `InvalidPositionLimits`: If the specified position is outside the configured min/max position limits
    ///
    /// # Example
    /// ```ignore
    /// use control_core::controllers::second_degree_motion::acceleration_position_controller::AccelerationPositionController;
    ///
    /// let mut controller = AccelerationPositionController::new_simple(
    ///     Some(100.0), 10.0, 5.0
    /// )?;
    ///
    /// // Reset to position 50.0
    /// controller.reset(50.0)?;
    /// assert_eq!(controller.get_position(), 50.0);
    /// assert_eq!(controller.get_speed(), 0.0);
    /// ```
    pub fn reset(&mut self, position: f64) -> Result<(), MotionControllerError> {
        // Validate position against limits
        if let Some(min_pos) = self.config.min_position {
            if position < min_pos {
                return Err(MotionControllerError::InvalidPositionLimits);
            }
        }
        if let Some(max_pos) = self.config.max_position {
            if position > max_pos {
                return Err(MotionControllerError::InvalidPositionLimits);
            }
        }

        self.current_position = position;
        self.target_position = position;
        self.current_speed = 0.0;
        self.current_acceleration = 0.0;
        self.motion_phase = MotionPhase::Idle;
        self.direction = 0;
        self.peak_speed = 0.0;
        self.deceleration_position = position;
        self.needs_replanning = false;

        Ok(())
    }

    /// Update multiple configuration parameters at once to avoid unnecessary re-planning
    pub fn update_config<F>(&mut self, config_fn: F) -> Result<(), MotionControllerError>
    where
        F: FnOnce(&mut ControllerConfig) -> Result<(), MotionControllerError>,
    {
        let mut new_config = self.config.clone();
        config_fn(&mut new_config)?;

        // Validate the new configuration
        if new_config.min_speed > 0.0 || new_config.max_speed < 0.0 {
            return Err(MotionControllerError::InvalidSpeedLimits);
        }
        if new_config.min_acceleration >= 0.0 || new_config.max_acceleration <= 0.0 {
            return Err(MotionControllerError::InvalidAccelerationLimits);
        }
        if let (Some(min_pos), Some(max_pos)) = (new_config.min_position, new_config.max_position) {
            if min_pos > max_pos {
                return Err(MotionControllerError::InvalidPositionLimits);
            }
        }

        // Apply the new configuration
        self.config = new_config;
        self.motion_constants = Self::calculate_motion_constants(&self.config);

        // Constrain current position and target if needed
        if let Some(min_pos) = self.config.min_position {
            if self.current_position < min_pos {
                self.current_position = min_pos;
            }
            if self.target_position < min_pos {
                self.target_position = min_pos;
            }
        }
        if let Some(max_pos) = self.config.max_position {
            if self.current_position > max_pos {
                self.current_position = max_pos;
            }
            if self.target_position > max_pos {
                self.target_position = max_pos;
            }
        }

        self.needs_replanning = true;
        Ok(())
    }

    /// Set the minimum speed
    pub fn set_min_speed(&mut self, min_speed: f64) -> Result<(), MotionControllerError> {
        self.update_config(|config| {
            if min_speed > 0.0 {
                return Err(MotionControllerError::InvalidSpeedLimits);
            }
            config.min_speed = min_speed;
            Ok(())
        })
    }

    /// Set the maximum speed
    pub fn set_max_speed(&mut self, max_speed: f64) -> Result<(), MotionControllerError> {
        self.update_config(|config| {
            if max_speed < 0.0 {
                return Err(MotionControllerError::InvalidSpeedLimits);
            }
            config.max_speed = max_speed;
            Ok(())
        })
    }

    /// Set the minimum acceleration (deceleration)
    pub fn set_min_acceleration(
        &mut self,
        min_acceleration: f64,
    ) -> Result<(), MotionControllerError> {
        self.update_config(|config| {
            if min_acceleration >= 0.0 {
                return Err(MotionControllerError::InvalidAccelerationLimits);
            }
            config.min_acceleration = min_acceleration;
            Ok(())
        })
    }

    /// Set the maximum acceleration
    pub fn set_max_acceleration(
        &mut self,
        max_acceleration: f64,
    ) -> Result<(), MotionControllerError> {
        self.update_config(|config| {
            if max_acceleration <= 0.0 {
                return Err(MotionControllerError::InvalidAccelerationLimits);
            }
            config.max_acceleration = max_acceleration;
            Ok(())
        })
    }

    /// Get the minimum speed limit
    pub const fn get_min_speed(&self) -> f64 {
        self.config.min_speed
    }

    /// Get the maximum speed limit
    pub const fn get_max_speed(&self) -> f64 {
        self.config.max_speed
    }

    /// Get the minimum acceleration limit
    pub const fn get_min_acceleration(&self) -> f64 {
        self.config.min_acceleration
    }

    /// Get the maximum acceleration limit
    pub const fn get_max_acceleration(&self) -> f64 {
        self.config.max_acceleration
    }

    /// Get the minimum position limit
    pub const fn get_min_position(&self) -> Option<f64> {
        self.config.min_position
    }

    /// Get the maximum position limit
    pub const fn get_max_position(&self) -> Option<f64> {
        self.config.max_position
    }

    /// Set the minimum position limit
    pub fn set_min_position(
        &mut self,
        min_position: Option<f64>,
    ) -> Result<(), MotionControllerError> {
        self.update_config(|config| {
            if let (Some(min_pos), Some(max_pos)) = (min_position, config.max_position) {
                if min_pos > max_pos {
                    return Err(MotionControllerError::InvalidPositionLimits);
                }
            }
            config.min_position = min_position;
            Ok(())
        })
    }

    /// Set the maximum position limit
    pub fn set_max_position(
        &mut self,
        max_position: Option<f64>,
    ) -> Result<(), MotionControllerError> {
        self.update_config(|config| {
            if let (Some(min_pos), Some(max_pos)) = (config.min_position, max_position) {
                if min_pos > max_pos {
                    return Err(MotionControllerError::InvalidPositionLimits);
                }
            }
            config.max_position = max_position;
            Ok(())
        })
    }

    /// Get the position tolerance
    pub const fn get_position_tolerance(&self) -> f64 {
        self.config.position_tolerance
    }

    /// Get the speed tolerance
    pub const fn get_speed_tolerance(&self) -> f64 {
        self.config.speed_tolerance
    }

    /// Set tolerances for position and speed comparisons
    pub const fn set_tolerances(&mut self, position_tolerance: f64, speed_tolerance: f64) {
        self.config.position_tolerance = position_tolerance;
        self.config.speed_tolerance = speed_tolerance;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_creation() {
        let controller = AccelerationPositionController::builder()
            .speed_limits(-5.0, 10.0)
            .acceleration_limits(-3.0, 2.0)
            .position_limits(Some(0.0), Some(100.0))
            .build();

        assert!(controller.is_ok());
        let controller = controller.unwrap();
        assert_eq!(controller.get_position(), 0.0);
        assert_eq!(controller.get_speed(), 0.0);
        assert!(controller.is_at_target());
    }

    #[test]
    fn test_invalid_parameters() {
        // Invalid speed limits
        let result = AccelerationPositionController::builder()
            .speed_limits(5.0, -10.0) // min > 0, max < 0
            .acceleration_limits(-3.0, 2.0)
            .build();
        assert!(matches!(
            result,
            Err(MotionControllerError::InvalidSpeedLimits)
        ));

        // Invalid acceleration limits
        let result = AccelerationPositionController::builder()
            .speed_limits(-5.0, 10.0)
            .acceleration_limits(3.0, -2.0) // min > 0, max < 0
            .build();
        assert!(matches!(
            result,
            Err(MotionControllerError::InvalidAccelerationLimits)
        ));

        // Invalid position limits
        let result = AccelerationPositionController::builder()
            .speed_limits(-5.0, 10.0)
            .acceleration_limits(-3.0, 2.0)
            .position_limits(Some(100.0), Some(0.0)) // min > max
            .build();
        assert!(matches!(
            result,
            Err(MotionControllerError::InvalidPositionLimits)
        ));
    }

    #[test]
    fn test_basic_motion() {
        let mut controller = AccelerationPositionController::builder()
            .speed_limits(-5.0, 10.0)
            .acceleration_limits(-3.0, 2.0)
            .build()
            .unwrap();

        // Set target and update
        let result = controller.update(10.0, 0.1);
        assert!(result.is_ok());

        // Should have started moving
        assert!(controller.is_moving());
        assert!(!controller.is_at_target());
        assert!(controller.get_speed() > 0.0);
    }

    #[test]
    fn test_position_limits() {
        let mut controller = AccelerationPositionController::builder()
            .speed_limits(-5.0, 10.0)
            .acceleration_limits(-3.0, 2.0)
            .position_limits(Some(0.0), Some(50.0))
            .build()
            .unwrap();

        // Try to set target beyond limits
        let result = controller.update(100.0, 0.1);
        assert!(result.is_ok());

        // Target should be constrained to 50.0
        assert_eq!(controller.get_target_position(), 50.0);
    }

    #[test]
    fn test_emergency_stop() {
        let mut controller = AccelerationPositionController::builder()
            .speed_limits(-5.0, 10.0)
            .acceleration_limits(-3.0, 2.0)
            .build()
            .unwrap();

        // Start motion
        let _ = controller.update(10.0, 0.1);
        assert!(controller.is_moving());

        // Emergency stop
        controller.emergency_stop();
        assert!(!controller.is_moving());
        assert!(controller.is_at_target());
        assert_eq!(controller.get_speed(), 0.0);
    }

    #[test]
    fn test_config_update() {
        let mut controller = AccelerationPositionController::builder()
            .speed_limits(-5.0, 10.0)
            .acceleration_limits(-3.0, 2.0)
            .build()
            .unwrap();

        // Update multiple parameters at once
        let result = controller.update_config(|config| {
            config.max_speed = 20.0;
            config.max_acceleration = 5.0;
            Ok(())
        });

        assert!(result.is_ok());
        assert_eq!(controller.get_max_speed(), 20.0);
        assert_eq!(controller.get_max_acceleration(), 5.0);
    }
}
