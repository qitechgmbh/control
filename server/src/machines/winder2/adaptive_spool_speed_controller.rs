use crate::machines::winder2::{
    clamp_revolution::clamp_revolution_uom, filament_tension::FilamentTensionCalculator,
    puller_speed_controller::PullerSpeedController,
};

use super::{clamp_revolution::Clamping, tension_arm::TensionArm};
use control_core::{
    controllers::{
        first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController,
        second_degree_motion::acceleration_position_controller::MotionControllerError,
    },
    helpers::{
        interpolation::{interpolate_hinge, scale},
        moving_time_window::MovingTimeWindow,
    },
    uom_extensions::angular_acceleration::revolution_per_minute_per_second,
};
use std::time::Instant;
use tracing::info;
use uom::{
    ConstZero,
    si::{
        angle::degree,
        angular_acceleration::radian_per_second_squared,
        angular_velocity::{radian_per_second, revolution_per_minute},
        f64::{Angle, AngularAcceleration, AngularVelocity},
        velocity::meter_per_second,
    },
};

#[derive(Debug)]
pub struct AdaptiveSpoolSpeedController {
    /// Current speed in
    last_speed: AngularVelocity,
    /// Whether the speed controller is enabled or not
    enabled: bool,
    /// Acceleration controller to dampen speed change
    acceleration_controller: AngularAccelerationSpeedController,
    /// Filament tension calculator
    filament_calc: FilamentTensionCalculator,
    /// Unit is angular velocity in rad/s
    speed_time_window: MovingTimeWindow<f64>,
    /// Learned speed factor parameter for gradient descent
    speed_factor: f64,
    /// Learning rate for gradient descent (per second)
    learning_rate: f64,
    /// Error history for gradient calculation
    filament_tension_error_history: MovingTimeWindow<f64>,
    /// Last time the speed factor was updated for time-aware learning
    last_speed_factor_update: Option<Instant>,
}

impl AdaptiveSpoolSpeedController {
    /// Parameters:
    /// - `min_speed`: Minimum speed
    /// - `max_speed`: Maximum speed  
    /// - `acceleration`: Acceleration
    /// - `deceleration`: Deceleration (preferably negative)
    pub fn new() -> Self {
        let max_speed = AngularVelocity::new::<revolution_per_minute>(150.0);

        Self {
            last_speed: AngularVelocity::ZERO,
            enabled: false,
            acceleration_controller: AngularAccelerationSpeedController::new(
                Some(AngularVelocity::ZERO),
                Some(max_speed),
                -AngularAcceleration::ZERO, // Will be dynamically adjusted
                AngularAcceleration::ZERO,  // Will be dynamically adjusted
                AngularVelocity::ZERO,
            ),
            filament_calc: FilamentTensionCalculator::new(
                Angle::new::<degree>(90.0),
                Angle::new::<degree>(20.0),
            ),
            speed_time_window: MovingTimeWindow::new(
                std::time::Duration::from_secs(5),
                10, // max samples
            ),
            speed_factor: 1.0,
            learning_rate: 0.1,
            filament_tension_error_history: MovingTimeWindow::new(
                std::time::Duration::from_secs(10),
                10, // max samples
            ),
            last_speed_factor_update: None,
        }
    }

    /// Calculates the desired speed based on the tension arm angle.
    ///
    /// If the arm is over it's maximum angle, the speed is set to the minimum speed.
    /// If the arm is under it's minimum angle, the speed is set to the maximum speed.
    /// If the arm is within the range, the speed is interpolated between the minimum and maximum speed based on the tension arm angle.
    ///
    /// Parameters:
    /// - `t`: The current time.
    /// - `tension_arm`: A reference to the `TensionArm` instance that provides the angle of the tension arm.
    ///
    /// Returns:
    /// - speed
    fn speed_smart(
        &mut self,
        t: Instant,
        tension_arm: &TensionArm,
        puller_speed_controller: &PullerSpeedController,
    ) -> AngularVelocity {
        let min_speed = AngularVelocity::ZERO;

        // Convert puller speed to angular velocity using constant factor * learned speed factor
        // The constant factor provides a baseline, while the learned factor adapts to keep tension optimal
        let total_speed_factor = 50.0 * self.speed_factor;
        let max_speed = AngularVelocity::new::<radian_per_second>(
            puller_speed_controller.last_speed.get::<meter_per_second>() * total_speed_factor,
        );

        // calculate filament tension
        let tension_arm_angle = tension_arm.get_angle();
        let tension_arm_revolution = clamp_revolution_uom(
            tension_arm_angle,
            // inverted because min angle is max tension
            self.filament_calc.get_max_angle(),
            self.filament_calc.get_min_angle(),
        );

        match tension_arm_revolution.1 {
            Clamping::Min => return min_speed,
            Clamping::Max => return min_speed,
            _ => {}
        };

        let filament_tension = self
            .filament_calc
            .calc_filament_tension(tension_arm_revolution.0);

        // move filament tension 0.7 to 0.5
        let filament_tension_hinged = interpolate_hinge(filament_tension, 0.7, 0.5);
        self.update_speed_factor(filament_tension_hinged, t);

        // interpolate speed linear
        let filament_tension_inverted = 1.0 - filament_tension;
        let speed = AngularVelocity::new::<radian_per_second>(scale(
            filament_tension_inverted,
            min_speed.get::<radian_per_second>(),
            max_speed.get::<radian_per_second>(),
        ));

        // save speed
        return speed;
    }

    /// Accelerates the speed using the acceleration controller.
    ///
    /// Parameters:
    /// - `speed`: The current speed
    /// - `t`: The current time.
    ///
    /// Returns:
    /// - The new speed after applying acceleration.
    fn accelerate_speed_smart(&mut self, speed: AngularVelocity, t: Instant) -> AngularVelocity {
        // The min/mac acceleration depends on the max speed of the last 5secs or the target speed (whatever is higher)
        let acceleration = AngularAcceleration::new::<radian_per_second_squared>(
            self.speed_time_window
                .max()
                .abs()
                .max(speed.get::<radian_per_second>().abs())
                // The magic factor is dependent on the scceleration settings on the puller speed controller to reduce oscillation
                * 1.0,
        );

        // Set the acceleration to the controller
        self.acceleration_controller
            .set_max_acceleration(acceleration);
        self.acceleration_controller
            .set_min_acceleration(-acceleration);

        let new_speed = self.acceleration_controller.update(speed, t);

        // add new speed to the time window
        self.speed_time_window
            .update(new_speed.get::<radian_per_second>(), t);

        return new_speed;
    }

    /// Clamps the speed to the defined minimum and maximum speed.
    ///
    /// Parameters:
    /// - `speed`: The speed to be clamped.
    ///
    /// Returns:
    /// - The clamped speed, ensuring it is within the range of `min_speed` and `max_speed`.
    fn clamp_speed(&mut self, speed: AngularVelocity) -> AngularVelocity {
        let min_speed = AngularVelocity::ZERO;
        let max_speed = AngularVelocity::new::<revolution_per_minute>(600.0);

        if speed < min_speed {
            return AngularVelocity::ZERO;
        } else if speed > max_speed {
            return max_speed;
        } else {
            return speed;
        }
    }

    fn update_speed_factor(&mut self, filament_tension: f64, t: Instant) {
        let delta_t = match self.last_speed_factor_update {
            Some(last_update) => t.duration_since(last_update).as_secs_f64(),
            None => {
                // First call, initialize and return early
                self.last_speed_factor_update = Some(t);
                return;
            }
        };

        // 0.5 is our proportional gain
        let kp = -0.5 * delta_t;

        // Target tension is 0.5
        let error = filament_tension - 0.5;

        let change = error * kp;

        let new_speed_factor = self.speed_factor + change;

        info!(
            "Updating speed factor: old={:.3}, change={:.8}, new={:.3}, tension={:.3}, delta_t={:.9}",
            self.speed_factor, change, new_speed_factor, filament_tension, delta_t
        );

        // Clamp the new speed factor to a reasonable range
        self.speed_factor = new_speed_factor.clamp(1.0, 5.0);

        // Update the timestamp only after making a change
        self.last_speed_factor_update = Some(t);
    }

    pub fn update_speed(
        &mut self,
        t: Instant,
        tension_arm: &TensionArm,
        puller_speed_controller: &PullerSpeedController,
    ) -> AngularVelocity {
        let speed = self.speed_smart(t, tension_arm, puller_speed_controller);
        let speed = match self.enabled {
            true => speed,
            false => AngularVelocity::ZERO,
        };
        let speed = self.accelerate_speed_smart(speed, t);

        // save speed before clamping or it will stay 0.0
        self.last_speed = speed;

        self.clamp_speed(speed)
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn reset(&mut self) {
        self.last_speed = AngularVelocity::ZERO;
        let _ = self.acceleration_controller.reset(AngularVelocity::ZERO);
        // Reset learned speed factor and error history
        self.speed_factor = 1.0;
        self.filament_tension_error_history =
            MovingTimeWindow::new(std::time::Duration::from_secs(10), 50);
        // Reset the last update time for gradient descent
        self.last_speed_factor_update = None;
    }

    fn update_acceleration(&mut self) -> Result<(), MotionControllerError> {
        // Set acceleration to 1/4 of the range between min and max speed
        // The spool will accelerate from min to max speed in 4 seconds
        let min_speed = self.get_min_speed();
        let max_speed = self.get_max_speed();
        let range = max_speed - min_speed;
        let acceleration = AngularAcceleration::new::<revolution_per_minute_per_second>(
            range.get::<revolution_per_minute>() / 4.0,
        );
        self.acceleration_controller
            .set_max_acceleration(acceleration);
        self.acceleration_controller
            .set_min_acceleration(-acceleration);
        Ok(())
    }

    pub fn set_max_speed(
        &mut self,
        max_speed: AngularVelocity,
    ) -> Result<(), MotionControllerError> {
        self.acceleration_controller.set_max_speed(Some(max_speed));
        self.update_acceleration()?;
        Ok(())
    }

    pub fn set_min_speed(
        &mut self,
        min_speed: AngularVelocity,
    ) -> Result<(), MotionControllerError> {
        self.acceleration_controller.set_min_speed(Some(min_speed));
        self.update_acceleration()?;
        Ok(())
    }

    pub fn get_max_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_max_speed()
            .expect("Max speed should be set")
    }

    pub fn get_min_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_min_speed()
            .expect("Min speed should be set")
    }

    pub fn get_speed(&self) -> AngularVelocity {
        self.last_speed
    }

    pub fn set_speed(&mut self, speed: AngularVelocity) {
        self.last_speed = speed;
        // Also update the acceleration controller's current speed to ensure smooth transitions
        let _ = self.acceleration_controller.reset(speed);
    }

    /// Get the current learned speed factor for monitoring/debugging
    pub fn get_learned_speed_factor(&self) -> f64 {
        self.speed_factor
    }

    /// Set the learning rate for gradient descent (default is 0.001)
    pub fn set_learning_rate(&mut self, rate: f64) {
        self.learning_rate = rate.max(0.0);
    }

    /// Get the current learning rate
    pub fn get_learning_rate(&self) -> f64 {
        self.learning_rate
    }
}
