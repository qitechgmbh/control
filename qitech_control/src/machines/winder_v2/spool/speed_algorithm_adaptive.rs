use std::time::Instant;

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_lib::units::AngularAcceleration;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Length;
use qitech_lib::units::Velocity;
use qitech_lib::units::angular_acceleration::radian_per_second_squared;
use qitech_lib::units::angular_velocity::radian_per_second;
use qitech_lib::units::length::centimeter;
use qitech_lib::units::length::meter;
use qitech_lib::units::velocity::meter_per_second;

use crate::controllers::first_degree_motion::AngularAccelerationSpeedController;
use crate::machines::winder_v2::spool::speed_controller::SpeedAlgorithmInput;
use crate::utils::interpolation::scale;

pub struct SpeedAlgorithmAdaptive {
    /// Timestamp of last max speed factor update, used for time-aware learning rate calculation
    pub(crate) last_max_speed_factor_update: Option<Instant>,

    /// Target normalized tension value (0.0-1.0) that the controller tries to maintain
    pub(crate) tension_target: ConfigProperty<f64>,

    /// Proportional control gain for adaptive learning (negative: higher tension reduces speed)
    pub(crate) radius_learning_rate: ConfigProperty<f64>,

    /// Speed multiplier when tension is at minimum (max speed factor)
    pub(crate) max_speed_multiplier: ConfigProperty<f64>,

    /// Base acceleration as a fraction of max possible speed (per second)
    pub(crate) acceleration_factor: ConfigProperty<f64>,

    /// Urgency multiplier for near-zero target speeds
    pub(crate) deacceleration_urgency_multiplier: ConfigProperty<f64>,

    /// Factor to control/calculate the feed speed based on filament tension
    pub(crate) speed_factor: Measurement<Length>,

    // --- controllers ----
    pub(crate) acceleration_controller: AngularAccelerationSpeedController,
}

// --- constants ---
impl SpeedAlgorithmAdaptive {
    const FACTOR_MIN: f64 = 4.25;
    const FACTOR_MAX: f64 = 20.0;

    const ACCELERATION_LIMIT_MIN: f64 = 0.5; // rad/s²
}

// --- init ---
impl SpeedAlgorithmAdaptive {
    pub fn init(ctx: &mut BuildContext) -> BuildResult<Self> {
        let tension_target = ctx
            .config::<f64>("spool.speed_controller.adaptive.tension_target")
            .default(0.7)
            .build()?;

        let radius_learning_rate = ctx
            .config::<f64>("spool.speed_controller.adaptive.radius_learning_rate")
            .default(0.5)
            .build()?;

        let max_speed_multiplier = ctx
            .config::<f64>("spool.speed_controller.adaptive.max_speed_multiplier")
            .default(4.0)
            .build()?;

        let acceleration_factor = ctx
            .config::<f64>("spool.speed_controller.adaptive.acceleration_factor")
            .default(0.2)
            .build()?;

        let deacceleration_urgency_multiplier = ctx
            .config::<f64>("spool.speed_controller.adaptive.deacceleration_urgency_multiplier")
            .default(15.0)
            .build()?;

        let speed_factor = ctx
            .measurement::<meter>("spool.speed_controller.adaptive.speed_factor")
            .initial(4.25)
            .build()?;

        let acceleration_controller = AngularAccelerationSpeedController::new(
            Some(AngularVelocity::ZERO),
            None,
            AngularAcceleration::ZERO, // Will be dynamically adjusted
            AngularAcceleration::ZERO, // Will be dynamically adjusted
            AngularVelocity::ZERO,
        );

        Ok(Self {
            speed_factor,
            last_max_speed_factor_update: None,
            tension_target,
            radius_learning_rate,
            max_speed_multiplier,
            acceleration_factor,
            deacceleration_urgency_multiplier,
            acceleration_controller,
        })
    }
}

// --- public interface ---
impl SpeedAlgorithmAdaptive {
    pub fn compute(&mut self, input: SpeedAlgorithmInput) -> AngularVelocity {
        let speed_max = self.compute_speed_max(input.puller_speed);

        let mut speed = if let Some(filament_tension) = input.filament_tension {
            self.update_speed_factor(filament_tension, Instant::now());

            AngularVelocity::new::<radian_per_second>(scale(
                1.0 - filament_tension,
                input.speed_min.get::<radian_per_second>(),
                input.speed_max.get::<radian_per_second>(),
            ))
        } else {
            AngularVelocity::ZERO
        };

        if !input.enabled {
            speed = AngularVelocity::ZERO;
        }

        self.accelerate_speed(Instant::now(), speed, speed_max)
    }
}

// --- utils ---
impl SpeedAlgorithmAdaptive {
    fn compute_speed_max(&self, puller_speed: Velocity) -> AngularVelocity {
        AngularVelocity::new::<radian_per_second>(
            (puller_speed.get::<meter_per_second>() / self.speed_factor.get_as::<meter>())
                * self.max_speed_multiplier.get(),
        )
    }

    fn update_speed_factor(&mut self, filament_tension: f64, t: Instant) {
        let delta_t = match self.last_max_speed_factor_update {
            Some(last_update) => t.duration_since(last_update).as_secs_f64(),
            None => {
                // First call, initialize and return early
                self.last_max_speed_factor_update = Some(t);
                return;
            }
        };

        // positive error means too much tension, so we reduce speed
        // negative error means too little tension, so we increase speed
        let tension_error = filament_tension - self.tension_target.get();

        // Calculate proportional control adjustment
        let proportional_gain = self.radius_learning_rate.get() * delta_t;
        let factor_change = tension_error * proportional_gain;

        // Update the speed factor directly
        let new_factor = (self.speed_factor.get_as::<centimeter>() + factor_change)
            .clamp(Self::FACTOR_MIN, Self::FACTOR_MAX);

        self.speed_factor.set_as::<centimeter>(new_factor);
        self.last_max_speed_factor_update = Some(t);
    }

    fn accelerate_speed(
        &mut self,
        now: Instant,
        speed_target: AngularVelocity,
        speed_max: AngularVelocity,
    ) -> AngularVelocity {
        let target_speed_rad_s = speed_target.get::<radian_per_second>();
        let max_speed_rad_s = speed_max.get::<radian_per_second>();

        // Base acceleration proportional to current max operating speed
        let base_acceleration = max_speed_rad_s * self.acceleration_factor.get();

        // Simple urgency factor - dramatically increases near zero
        let urgency_factor = if target_speed_rad_s.abs() < 0.1 {
            self.deacceleration_urgency_multiplier.get() * (1.0 / (target_speed_rad_s.abs() + 0.01))
        } else {
            1.0
        };

        // Final acceleration limit
        let acceleration_limit =
            (base_acceleration * urgency_factor).max(Self::ACCELERATION_LIMIT_MIN);

        let acceleration =
            AngularAcceleration::new::<radian_per_second_squared>(acceleration_limit);

        // Update acceleration controller limits
        self.acceleration_controller
            .set_max_acceleration(acceleration);
        self.acceleration_controller
            .set_min_acceleration(-acceleration);
        self.acceleration_controller.update(speed_target, now)
    }
}
