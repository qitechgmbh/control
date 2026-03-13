use std::time::Duration;

use control_core::{
    controllers::first_degree_motion::AngularAccelerationSpeedController,
    helpers::{
        interpolation::{interpolate_exponential, scale},
        moving_time_window::MovingTimeWindow,
    },
};
use units::{
    AngularAcceleration, AngularVelocity, ConstZero,
    angular_acceleration::radian_per_second_squared, angular_velocity::radian_per_second,
};

use crate::types::{Bounds, ExceededBound};

use super::AlgorithmInput;

pub struct MinMaxSpeedAlgorithm {
    constants: Constants,
    variables: Variables,
    variables_default: Variables,
    speed_time_window: MovingTimeWindow<AngularVelocity>,
    acceleration_controller: AngularAccelerationSpeedController,
}

// getters + setters
impl MinMaxSpeedAlgorithm {
    pub fn speed_min(&self) -> AngularVelocity {
        self.variables.speed_min
    }

    pub fn speed_max(&self) -> AngularVelocity {
        self.variables.speed_max
    }

    pub fn set_speed_min(&mut self, min: AngularVelocity) -> Result<(), VariableSetError> {
        self.set_speed_limits(min, self.variables.speed_max)
    }

    pub fn set_speed_max(&mut self, max: AngularVelocity) -> Result<(), VariableSetError> {
        self.set_speed_limits(self.variables.speed_min, max)
    }

    pub fn set_speed_limits(
        &mut self,
        min: AngularVelocity,
        max: AngularVelocity,
    ) -> Result<(), VariableSetError> {
        Self::validate_speed_limits(self.constants.speed_limits_hard, min, max)?;

        self.acceleration_controller.set_min_speed(Some(min));
        self.acceleration_controller.set_max_speed(Some(max));

        self.variables.speed_min = min;
        self.variables.speed_max = max;

        Ok(())
    }
}

// public interface
impl MinMaxSpeedAlgorithm {
    pub fn new(constants: Constants, variables: Variables) -> Result<Self, VariableSetError> {
        Self::validate_speed_limits(
            constants.speed_limits_hard,
            variables.speed_min,
            variables.speed_max,
        )?;

        let speed_time_window = MovingTimeWindow::new(
            constants.speed_window_duration,
            constants.speed_window_samples_max,
        );

        let acceleration_controller = AngularAccelerationSpeedController::new(
            Some(variables.speed_min),
            Some(variables.speed_max),
            -AngularAcceleration::ZERO, // Will be dynamically adjusted
            AngularAcceleration::ZERO,  // Will be dynamically adjusted
            variables.speed_min,
        );

        Ok(Self {
            constants,
            variables,
            variables_default: variables,
            speed_time_window,
            acceleration_controller,
        })
    }

    pub fn compute_speed(&mut self, input: AlgorithmInput) -> AngularVelocity {
        let speed_raw = match input.enabled {
            true => self.compute_raw_speed(input.filament_tension),
            false => self.speed_min(),
        };

        self.update_acceleration_limits(speed_raw);

        let speed = self.acceleration_controller.update(speed_raw, input.t);

        // add new speed to the time window
        self.speed_time_window.update(speed, input.t);

        speed
    }

    pub fn restore_defaults(&mut self) {
        self.variables = self.variables_default;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.acceleration_controller
            .reset(self.variables_default.speed_min);
    }
}

// utils
impl MinMaxSpeedAlgorithm {
    fn compute_raw_speed(&self, filament_tension: f64) -> AngularVelocity {
        let filament_tension_inverted = 1.0 - filament_tension;

        let filament_tension_exponential = interpolate_exponential(filament_tension_inverted, 2.0);

        //TODO: scale_uom so this dumb conversion doesn't happen
        AngularVelocity::new::<radian_per_second>(scale(
            filament_tension_exponential,
            self.speed_min().get::<radian_per_second>(),
            self.speed_max().get::<radian_per_second>(),
        ))
    }

    fn update_acceleration_limits(&mut self, speed_target: AngularVelocity) {
        // highest achieved speed in time window
        let speed_peak = self.speed_time_window.max().abs();

        // highest speed between achieved and current
        let speed_max = speed_peak.max(speed_target.abs());

        let acceleration = AngularAcceleration::new::<radian_per_second_squared>(
            speed_max.get::<radian_per_second>() * self.constants.magic_factor,
        );

        self.acceleration_controller
            .set_max_acceleration(acceleration);
        self.acceleration_controller
            .set_min_acceleration(-acceleration);
    }

    fn validate_speed_limits(
        limits: Bounds<AngularVelocity>,
        min: AngularVelocity,
        max: AngularVelocity,
    ) -> Result<(), VariableSetError> {
        for v in [min, max] {
            if let Some(exceeded_bound) = limits.check(v) {
                return Err(VariableSetError::OutOfBounds(exceeded_bound));
            }
        }

        if min > max {
            return Err(VariableSetError::MinGreaterThanMax);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Constants {
    /// Hard limits for speed
    pub speed_limits_hard: Bounds<AngularVelocity>,

    /// The magic factor is dependent on the acceleration settings on the
    /// puller speed controller to reduce oscillation
    pub magic_factor: f64,

    /// Expiry duration of samples
    pub speed_window_duration: Duration,

    /// Allowed number of samples
    pub speed_window_samples_max: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Variables {
    pub speed_min: AngularVelocity,
    pub speed_max: AngularVelocity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableSetError {
    OutOfBounds(ExceededBound),
    MinGreaterThanMax,
}
