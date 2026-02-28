use std::time::Instant;

use units::{
    ConstZero,
    f64::*,
    angle::degree,
    angular_acceleration::{radian_per_second_squared, revolution_per_minute_per_second},
    angular_velocity::{radian_per_second, revolution_per_minute, revolution_per_second},
    length::meter,
    velocity::meter_per_second,
};

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

use crate::{helpers::{Clamp, clamp_revolution_uom}, winder2::devices::{Puller, spool::speed_controller::SpeedController}};
use crate::winder2::devices::TensionArm;
use super::helpers::FilamentTensionCalculator;

#[derive(Debug)]
pub struct MinMaxSpeedController
{
    /// Current speed in
    speed: AngularVelocity,
    /// Whether the speed controller is enabled or not
    enabled: bool,
    /// Acceleration controller to dampen speed change
    acceleration_controller: AngularAccelerationSpeedController,
    /// Filament tension calculator
    filament_calc: FilamentTensionCalculator,
    /// Unit is angular velocity in rad/s
    speed_time_window: MovingTimeWindow<f64>,
}

impl MinMaxSpeedController 
{
    pub fn new() -> Self 
    {
        let max_speed = AngularVelocity::new::<revolution_per_minute>(150.0);

        Self {
            speed: AngularVelocity::ZERO,
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

impl MinMaxSpeedController 
{
    fn speed_raw(&mut self, _t: Instant, tension_arm_angle: Angle) -> AngularVelocity
    {
        let min_speed = AngularVelocity::ZERO;
        let max_speed = self.max_speed();

        let tension_arm_revolution = clamp_revolution_uom(
            tension_arm_angle,
            // inverted because min angle is max tension
            self.filament_calc.get_max_angle(),
            self.filament_calc.get_min_angle(),
        );

        // if value was clamped return min speed.
        // why? Idk ask guy who left
        if matches!(tension_arm_revolution.clamp, Clamp::Min | Clamp::Max) {
            return min_speed;
        }

        let filament_tension = self
            .filament_calc
            .calc_filament_tension(tension_arm_revolution.value);

        let filament_tension_inverted = 1.0 - filament_tension;

        let filament_tension_exponential = interpolate_exponential(filament_tension_inverted, 2.0);

        AngularVelocity::new::<radian_per_second>(scale(
            filament_tension_exponential,
            min_speed.get::<radian_per_second>(),
            max_speed.get::<radian_per_second>(),
        ))
    }

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

impl MinMaxSpeedController 
{
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

    pub fn max_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_max_speed()
            .unwrap_or(AngularVelocity::ZERO)
    }

    pub fn min_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_min_speed()
            .unwrap_or(AngularVelocity::ZERO)
    }

    pub fn get_speed(&self) -> AngularVelocity {
        self.speed
    }

    pub fn set_speed(&mut self, speed: AngularVelocity) {
        self.speed = speed;
        // Also update the acceleration controller's current speed to ensure smooth transitions
        self.acceleration_controller.reset(speed);
    }

    /// derive the radius from the puller speed and the current angular speed
    pub fn get_radius(&self, puller: &Puller) -> Length 
    {
        let puller_speed  = puller.target_speed().get::<meter_per_second>();
        let angular_speed = self.speed.get::<revolution_per_second>();

        // Calculate the radius using the formula: radius = speed / angular_speed
        let radius = puller_speed / angular_speed;

        // Ensure the radius is a normal number, otherwise default to 0.0
        let radius = Some(radius).filter(|&n| n.is_normal()).unwrap_or(0.0);

        Length::new::<meter>(radius)
    }
}

impl SpeedController for MinMaxSpeedController
{
    fn speed(&self) -> AngularVelocity {
        self.speed
    }

    fn set_speed(&mut self, speed: AngularVelocity) {
        self.speed = speed;
        // Also update the acceleration controller's 
        // current speed to ensure smooth transitions
        self.acceleration_controller.reset(speed);
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn update_speed(
        &mut self, 
        t: Instant, 
        tension_arm: &TensionArm, 
        puller: &Puller
    ) -> AngularVelocity
    {
        _ = puller;

        let speed = self.speed_raw(t, tension_arm.get_angle());
        let speed = match self.enabled 
        {
            true  => speed,
            false => AngularVelocity::ZERO,
        };
        let speed = self.accelerate_speed(speed, t);

        // save speed before clamping or it will stay 0.0
        self.speed = speed;

        self.clamp_speed(speed)
    }
    
    fn reset(&mut self) {
        self.speed = AngularVelocity::ZERO;
        self.acceleration_controller.reset(AngularVelocity::ZERO);
    }
}