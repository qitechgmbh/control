use crate::winder2::{
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
        interpolation::{interpolate_exponential, scale},
        moving_time_window::MovingTimeWindow,
    },
};
use std::time::Instant;

use units::ConstZero;
use units::angle::degree;
use units::angular_acceleration::{radian_per_second_squared, revolution_per_minute_per_second};
use units::angular_velocity::{radian_per_second, revolution_per_minute, revolution_per_second};
use units::f64::*;
use units::length::meter;
use units::velocity::meter_per_second;

#[derive(Debug)]
pub struct MinMaxSpoolSpeedController {
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
}

impl Default for MinMaxSpoolSpeedController {
    fn default() -> Self {
        Self::new()
    }
}

impl MinMaxSpoolSpeedController {
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
        }
    }
}

impl MinMaxSpoolSpeedController {
    /// Helper method to get min speed without Option type
    fn min_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_min_speed()
            .unwrap_or(AngularVelocity::ZERO)
    }

    /// Helper method to get max speed without Option type  
    fn max_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_max_speed()
            .unwrap_or(AngularVelocity::new::<radian_per_second>(f64::INFINITY))
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
    fn speed_raw(&mut self, _t: Instant, tension_arm: &TensionArm) -> AngularVelocity {
        let min_speed = AngularVelocity::ZERO;

        // Convert puller speed to angular velocity using a magic factor
        // The factor is adjusted so that the tension arm is reasonably high when a standard spool is at its lowest diameter

        // Respect the configured max speed limits by taking the minimum of dynamic calculation and configured max
        let max_speed = self.max_speed();

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

        let filament_tension_inverted = 1.0 - filament_tension;

        // use exponetial interpolation to make the speed change more sensitive in the lower range
        let filament_tension_exponential = interpolate_exponential(filament_tension_inverted, 2.0);

        // interpolate speed linear

        // save speed
        AngularVelocity::new::<radian_per_second>(scale(
            filament_tension_exponential,
            min_speed.get::<radian_per_second>(),
            max_speed.get::<radian_per_second>(),
        ))
    }

    /// Accelerates the speed using the acceleration controller.
    ///
    /// Parameters:
    /// - `speed`: The current speed
    /// - `t`: The current time.
    ///
    /// Returns:
    /// - The new speed after applying acceleration.
    fn accelerate_speed(&mut self, speed: AngularVelocity, t: Instant) -> AngularVelocity {
        // The min/mac acceleration depends on the max speed of the last 5secs or the target speed (whatever is higher)
        let acceleration = AngularAcceleration::new::<radian_per_second_squared>(
            self.speed_time_window
                .max()
                .abs()
                .max(speed.get::<radian_per_second>().abs())
                // The magic factor is dependent on the scceleration settings on the puller speed controller to reduce oscillation
                * 0.5,
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

        new_speed
    }

    /// Clamps the speed to the defined minimum and maximum speed.
    ///
    /// Parameters:
    /// - `speed`: The speed to be clamped.
    ///
    /// Returns:
    /// - The clamped speed, ensuring it is within the range of `min_speed` and `max_speed`.
    fn clamp_speed(&mut self, speed: AngularVelocity) -> AngularVelocity {
        let min_speed = self.min_speed();
        let max_speed = self.max_speed();

        if speed < min_speed {
            AngularVelocity::ZERO
        } else if speed > max_speed {
            max_speed
        } else {
            speed
        }
    }
}

impl MinMaxSpoolSpeedController {
    pub fn update_speed(&mut self, t: Instant, tension_arm: &TensionArm) -> AngularVelocity {
        let speed = self.speed_raw(t, tension_arm);
        let speed = match self.enabled {
            true => speed,
            false => AngularVelocity::ZERO,
        };
        let speed = self.accelerate_speed(speed, t);

        // save speed before clamping or it will stay 0.0
        self.last_speed = speed;

        self.clamp_speed(speed)
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn reset(&mut self) {
        self.last_speed = AngularVelocity::ZERO;
        self.acceleration_controller.reset(AngularVelocity::ZERO);
    }

    fn update_acceleration(&mut self) -> Result<(), MotionControllerError> {
        // Set acceleration to 1/4 of the range between min and max speed
        // The spool will accelerate from min to max speed in 4 seconds
        let min_speed = self.min_speed();
        let max_speed = self.max_speed();
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
        self.max_speed()
    }

    pub fn get_min_speed(&self) -> AngularVelocity {
        self.min_speed()
    }

    pub fn get_speed(&self) -> AngularVelocity {
        self.last_speed
    }

    pub fn set_speed(&mut self, speed: AngularVelocity) {
        self.last_speed = speed;
        // Also update the acceleration controller's current speed to ensure smooth transitions
        self.acceleration_controller.reset(speed);
    }

    /// derive the radius from the puller speed and the current angular speed
    pub fn get_radius(&self, puller_speed_controller: &PullerSpeedController) -> Length {
        let puller_speed: Velocity = puller_speed_controller.get_target_speed();
        let angular_speed: AngularVelocity = self.last_speed;

        // Calculate the radius using the formula: radius = speed / angular_speed
        let radius =
            puller_speed.get::<meter_per_second>() / angular_speed.get::<revolution_per_second>();

        // Ensure the radius is a normal number, otherwise default to 0.0
        let radius = Some(radius).filter(|&n| n.is_normal()).unwrap_or(0.0);

        Length::new::<meter>(radius)
    }
}
