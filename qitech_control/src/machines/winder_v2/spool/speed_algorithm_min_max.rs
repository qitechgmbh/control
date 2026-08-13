use std::time::Duration;
use std::time::Instant;

use qitech_lib::units::AngularAcceleration;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::ConstZero;
use qitech_lib::units::angular_acceleration::radian_per_second_squared;
use qitech_lib::units::angular_velocity::radian_per_second;

use crate::controllers::first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController;
use crate::machines::winder_v2::spool::speed_controller::SpeedAlgorithmInput;
use crate::utils::interpolation::interpolate_exponential;
use crate::utils::interpolation::scale;
use crate::utils::moving_time_window::MovingTimeWindow;

pub struct SpeedAlgorithmMinMax {
    pub(crate) speed_time_window: MovingTimeWindow<f64>, // in rad/s
    pub(crate) acceleration_controller: AngularAccelerationSpeedController,
}

impl SpeedAlgorithmMinMax {
    pub fn init() -> Self {
        let speed_time_window = MovingTimeWindow::new(std::time::Duration::from_secs(5), 10);

        let acceleration_controller = AngularAccelerationSpeedController::new(
            Some(AngularVelocity::ZERO),
            None,
            AngularAcceleration::ZERO,
            AngularAcceleration::ZERO,
            AngularVelocity::ZERO,
        );

        Self {
            speed_time_window,
            acceleration_controller,
        }
    }
}

impl SpeedAlgorithmMinMax {
    pub fn compute(&mut self, input: SpeedAlgorithmInput) -> AngularVelocity {
        let speed = match input.enabled {
            true => self.compute_raw(&input),
            false => AngularVelocity::ZERO,
        };

        self.accelerate(input.dt, speed)
    }

    fn accelerate(&mut self, dt: Duration, speed: AngularVelocity) -> AngularVelocity {
        _ = dt;

        // The min/max acceleration depends on the max speed of the last 5 secs or the target speed (whatever is higher)
        let acceleration = AngularAcceleration::new::<radian_per_second_squared>(
            self.speed_time_window
                .max()
                .abs()
                .max(speed.get::<radian_per_second>().abs())
                // The magic factor is dependent on the acceleration settings on the puller speed controller to reduce oscillation
                * 0.5,
        );

        // Set the acceleration to the controller
        self.acceleration_controller
            .set_max_acceleration(acceleration);
        self.acceleration_controller
            .set_min_acceleration(-acceleration);

        let now = Instant::now();
        let new_speed = self.acceleration_controller.update(speed, now);

        // add new speed to the time window
        self.speed_time_window
            .update(new_speed.get::<radian_per_second>(), now);

        new_speed
    }

    /// Calculates the desired speed based on the tension arm angle.
    /// If the arm is over it's maximum angle, the speed is set to the minimum speed.
    /// If the arm is under it's minimum angle, the speed is set to the maximum speed.
    /// If the arm is within the range, the speed is interpolated between the minimum and maximum speed based on the tension arm angle.
    fn compute_raw(&mut self, input: &SpeedAlgorithmInput) -> AngularVelocity {
        let min_speed = AngularVelocity::ZERO;

        let Some(filament_tension) = input.filament_tension else {
            // no value because out of bounds
            return min_speed;
        };

        let filament_tension_inverted = 1.0 - filament_tension;

        // use exponetial interpolation to make the speed change more sensitive in the lower range
        let filament_tension_exponential = interpolate_exponential(filament_tension_inverted, 2.0);

        // interpolate speed linear
        AngularVelocity::new::<radian_per_second>(scale(
            filament_tension_exponential,
            input.speed_min.get::<radian_per_second>(),
            input.speed_max.get::<radian_per_second>(),
        ))
    }
}
