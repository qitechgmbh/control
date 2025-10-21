use std::time::Instant;

use units::{
    acceleration::meter_per_second_squared,
    f64::{Acceleration, Length, Velocity},
    length::meter,
    velocity::meter_per_second,
};

use super::acceleration_position_controller::{
    AccelerationPositionController, MotionControllerError,
};

/// Linear Acceleration Position Controller with proper physical units
///
/// This controller provides linear motion control with proper SI units (meters, m/s, m/s²).
/// It wraps the core AccelerationPositionController and provides unit-typed interfaces
/// for position (Length), velocity (Velocity), and acceleration (Acceleration).
///
/// The controller manages smooth motion profiles with configurable limits and tolerances,
/// ensuring that position, speed, and acceleration constraints are respected during motion.
///
/// # Example
/// ```ignore
/// use units::{length::meter, velocity::meter_per_second, acceleration::meter_per_second_squared};
/// use units::f64::{Length, Velocity, Acceleration};
///
/// let position_limit = Length::new::<meter>(10.0);
/// let max_speed = Velocity::new::<meter_per_second>(2.0);
/// let max_acceleration = Acceleration::new::<meter_per_second_squared>(5.0);
///
/// let controller = LinearAccelerationPositionController::new_simple(
///     Some(position_limit),
///     max_speed,
///     max_acceleration,
/// ).unwrap();
/// ```
#[derive(Debug)]
pub struct LinearAccelerationPositionController {
    controller: AccelerationPositionController,
    last_update: Option<Instant>,
}

impl LinearAccelerationPositionController {
    /// Create a new linear position controller with acceleration limits
    ///
    /// # Parameters
    /// * `min_position` - Minimum position limit (optional)
    /// * `max_position` - Maximum position limit (optional)  
    /// * `min_speed` - Minimum speed limit (typically negative)
    /// * `max_speed` - Maximum speed limit (typically positive)
    /// * `min_acceleration` - Minimum acceleration limit (typically negative for deceleration)
    /// * `max_acceleration` - Maximum acceleration limit (typically positive)
    ///
    /// # Returns
    /// A new LinearAccelerationPositionController instance
    ///
    /// # Panics
    /// Panics if the underlying AccelerationPositionController cannot be created with the given parameters
    pub fn new(
        min_position: Option<Length>,
        max_position: Option<Length>,
        min_speed: Velocity,
        max_speed: Velocity,
        min_acceleration: Acceleration,
        max_acceleration: Acceleration,
    ) -> Self {
        Self {
            controller: AccelerationPositionController::new(
                min_speed.get::<meter_per_second>(),
                max_speed.get::<meter_per_second>(),
                min_acceleration.get::<meter_per_second_squared>(),
                max_acceleration.get::<meter_per_second_squared>(),
                min_position.map(|length| length.get::<meter>()),
                max_position.map(|length| length.get::<meter>()),
                1e-6, // position_tolerance
                1e-6, // speed_tolerance
            )
            .expect("Failed to create AccelerationPositionController"),
            last_update: None,
        }
    }

    /// Create a new simple linear acceleration position controller with symmetric limits
    ///
    /// This is a convenience constructor that creates symmetric limits around zero for
    /// position, speed, and acceleration. This is useful for most common use cases.
    ///
    /// # Parameters
    /// * `position` - Maximum position limit (creates [-position, +position] range if Some)
    /// * `speed` - Maximum speed limit (creates [-speed, +speed] range)
    /// * `acceleration` - Maximum acceleration limit (creates [-acceleration, +acceleration] range)
    ///
    /// # Returns
    /// A Result containing the new controller or a MotionControllerError
    ///
    /// # Errors
    /// Returns MotionControllerError if:
    /// - Speed or acceleration values are negative or zero
    /// - The underlying controller cannot be created with the calculated limits
    ///
    /// # Example
    /// ```ignore
    /// use units::{length::meter, velocity::meter_per_second, acceleration::meter_per_second_squared};
    /// use units::f64::{Length, Velocity, Acceleration};
    ///
    /// let position_limit = Length::new::<meter>(5.0);  // Creates [-5.0, +5.0] meter range
    /// let max_speed = Velocity::new::<meter_per_second>(1.0);  // Creates [-1.0, +1.0] m/s range
    /// let max_accel = Acceleration::new::<meter_per_second_squared>(2.0);  // Creates [-2.0, +2.0] m/s² range
    ///
    /// let controller = LinearAccelerationPositionController::new_simple(
    ///     Some(position_limit),
    ///     max_speed,
    ///     max_accel,
    /// )?;
    /// ```
    pub fn new_simple(
        position: Option<Length>,
        speed: Velocity,
        acceleration: Acceleration,
    ) -> Result<Self, MotionControllerError> {
        let controller = AccelerationPositionController::new_simple(
            position.map(|p| p.get::<meter>()),
            speed.get::<meter_per_second>(),
            acceleration.get::<meter_per_second_squared>(),
        )?;

        Ok(Self {
            controller,
            last_update: None,
        })
    }

    /// Update the controller with a new target position
    ///
    /// Calculates the next position in the motion profile based on the target position
    /// and elapsed time since the last update. The controller automatically manages
    /// acceleration and velocity to reach the target while respecting all limits.
    ///
    /// # Parameters
    /// * `target_position` - The desired target position
    /// * `t` - Current timestamp for calculating time delta
    ///
    /// # Returns
    /// The current position after applying the motion profile for this time step
    ///
    /// # Panics
    /// Panics if the underlying controller update fails (should not happen in normal operation)
    ///
    /// # Example
    /// ```ignore
    /// use std::time::Instant;
    /// use units::{length::meter, f64::Length};
    ///
    /// let mut controller = LinearAccelerationPositionController::new_simple(
    ///     Some(Length::new::<meter>(10.0)),
    ///     // ... other parameters
    /// )?;
    ///
    /// let target = Length::new::<meter>(5.0);
    /// let current_position = controller.update(target, Instant::now());
    /// ```
    pub fn update(&mut self, target_position: Length, t: Instant) -> Length {
        // Calculate dt from the last update
        let dt = if let Some(last_t) = self.last_update {
            t.duration_since(last_t).as_secs_f64()
        } else {
            0.0 // First update, no time has passed
        };
        self.last_update = Some(t);

        // Convert target to raw value, update controller, and convert result back to Length
        let target_raw = target_position.get::<meter>();
        let position_raw = self
            .controller
            .update(target_raw, dt)
            .expect("Failed to update controller");
        Length::new::<meter>(position_raw)
    }

    /// Get the current position
    ///
    /// Returns the controller's current position in the motion profile.
    ///
    /// # Returns
    /// Current position as a Length value
    pub fn get_position(&self) -> Length {
        Length::new::<meter>(self.controller.get_position())
    }

    /// Get the target position  
    ///
    /// Returns the current target position that the controller is trying to reach.
    ///
    /// # Returns
    /// Target position as a Length value
    pub fn get_target_position(&self) -> Length {
        Length::new::<meter>(self.controller.get_target_position())
    }

    /// Get the minimum position limit
    ///
    /// Returns the minimum position constraint, if one is set.
    ///
    /// # Returns
    /// Minimum position limit as an Option<Length>, None if no limit is set
    pub fn get_min_position(&self) -> Option<Length> {
        self.controller.get_min_position().map(Length::new::<meter>)
    }

    /// Get the maximum position limit
    ///
    /// Returns the maximum position constraint, if one is set.
    ///
    /// # Returns  
    /// Maximum position limit as an Option<Length>, None if no limit is set
    pub fn get_max_position(&self) -> Option<Length> {
        self.controller.get_max_position().map(Length::new::<meter>)
    }

    /// Set the minimum position limit
    ///
    /// Updates the minimum position constraint for the controller.
    ///
    /// # Parameters
    /// * `min_position` - New minimum position limit (None to remove limit)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., greater than the maximum position limit)
    pub fn set_min_position(
        &mut self,
        min_position: Option<Length>,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_position(min_position.map(|length| length.get::<meter>()))
    }

    /// Set the maximum position limit
    ///
    /// Updates the maximum position constraint for the controller.
    ///
    /// # Parameters
    /// * `max_position` - New maximum position limit (None to remove limit)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., less than the minimum position limit)
    pub fn set_max_position(
        &mut self,
        max_position: Option<Length>,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_position(max_position.map(|length| length.get::<meter>()))
    }

    /// Get the current speed
    ///
    /// Returns the controller's current velocity in the motion profile.
    ///
    /// # Returns
    /// Current velocity as a Velocity value
    pub fn get_speed(&self) -> Velocity {
        Velocity::new::<meter_per_second>(self.controller.get_speed())
    }

    /// Set the minimum speed
    ///
    /// Updates the minimum velocity constraint for the controller.
    ///
    /// # Parameters
    /// * `min_speed` - New minimum speed limit (typically negative for reverse motion)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., greater than the maximum speed limit)
    pub fn set_min_speed(&mut self, min_speed: Velocity) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_speed(min_speed.get::<meter_per_second>())
    }

    /// Set the maximum speed
    ///
    /// Updates the maximum velocity constraint for the controller.
    ///
    /// # Parameters
    /// * `max_speed` - New maximum speed limit (typically positive)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., less than the minimum speed limit or negative/zero value)
    pub fn set_max_speed(&mut self, max_speed: Velocity) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_speed(max_speed.get::<meter_per_second>())
    }

    /// Get the current acceleration
    ///
    /// Returns the controller's current acceleration in the motion profile.
    ///
    /// # Returns
    /// Current acceleration as an Acceleration value
    pub fn get_acceleration(&self) -> Acceleration {
        Acceleration::new::<meter_per_second_squared>(self.controller.get_acceleration())
    }

    /// Set the minimum acceleration
    ///
    /// Updates the minimum acceleration constraint for the controller.
    ///
    /// # Parameters
    /// * `min_acceleration` - New minimum acceleration limit (typically negative for deceleration)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., greater than the maximum acceleration limit)
    pub fn set_min_acceleration(
        &mut self,
        min_acceleration: Acceleration,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_min_acceleration(min_acceleration.get::<meter_per_second_squared>())
    }

    /// Set the maximum acceleration
    ///
    /// Updates the maximum acceleration constraint for the controller.
    ///
    /// # Parameters
    /// * `max_acceleration` - New maximum acceleration limit (typically positive)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the new limit would be invalid
    /// (e.g., less than the minimum acceleration limit or negative/zero value)
    pub fn set_max_acceleration(
        &mut self,
        max_acceleration: Acceleration,
    ) -> Result<(), MotionControllerError> {
        self.controller
            .set_max_acceleration(max_acceleration.get::<meter_per_second_squared>())
    }

    /// Reset the controller to a new position and velocity
    ///
    /// This resets all internal state of the controller to start fresh from a new state.
    /// The controller will reset its position, clear any accumulated motion profile state,
    /// and set the target position to the current position.
    ///
    /// # Parameters
    /// * `position` - The new current position to reset to
    ///
    /// # Returns
    /// Result indicating success or failure of the reset operation
    ///
    /// # Errors
    /// Returns MotionControllerError if the position is outside the configured limits
    ///
    /// # Example
    /// ```ignore
    /// use units::{length::meter, velocity::meter_per_second, f64::{Length, Velocity}};
    ///
    /// let new_position = Length::new::<meter>(0.0);
    ///
    /// controller.reset(new_position)?;
    /// ```
    pub fn reset(&mut self, position: Length) -> Result<(), MotionControllerError> {
        self.controller.reset(position.get::<meter>())?;
        self.last_update = None;
        Ok(())
    }
}
