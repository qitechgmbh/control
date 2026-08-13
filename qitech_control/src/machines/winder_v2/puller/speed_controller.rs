use std::time::Duration;

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Length;
use qitech_lib::units::Velocity;
use qitech_lib::units::length::meter;
use qitech_lib::units::length::millimeter;
use qitech_lib::units::velocity::meter_per_minute;
use qitech_lib::units::velocity::meter_per_second;

use crate::machines::winder_v2::WinderV1;
use crate::machines::winder_v2::puller::SpeedControlAlgorithm;
use crate::machines::winder_v2::types::LaserSubscription;

pub struct SpeedController {
    // --- config ---
    algorithm: ConfigProperty<SpeedControlAlgorithm>,
    speed_desired: ConfigProperty<Velocity>,

    // --- measurements ---
    speed_target: Measurement<Velocity>,

    // --- speed algorithms ---
    sa_adaptive: SpeedAlgorithmAdaptive,
}

// --- init ---
impl SpeedController {
    pub fn new<const VARIANT: usize>(ctx: &mut BuildContext) -> BuildResult<Self> {
        Ok(Self {
            algorithm: ctx
                .config::<SpeedControlAlgorithm>("puller.speed_controller.algorithm")
                .on_external_changed(|m: &mut WinderV1<VARIANT>| {
                    m.puller.speed_controller.sa_adaptive.reset_modulation();
                    Ok(())
                })
                .build()?,

            speed_desired: ctx
                .config::<meter_per_minute>("puller.speed_controller.speed_desired")
                .build()?,

            speed_target: ctx
                .measurement::<meter_per_minute>("puller.speed_controller.speed_target")
                .build()?,

            sa_adaptive: SpeedAlgorithmAdaptive::new(ctx)?,
        })
    }
}

// --- public interface ---
impl SpeedController {
    pub fn update(
        &mut self,
        dt: Duration,
        speed_prev: Velocity,
        laser_subscription: Option<&LaserSubscription>,
    ) {
        if let Some(laser) = laser_subscription {
            self.sa_adaptive.update(
                dt,
                speed_prev,
                laser.diameter.get_as::<millimeter>(),
                laser.diameter_target.get_as::<millimeter>(),
            );
        }

        let speed = match self.algorithm.get() {
            SpeedControlAlgorithm::Direct => self.speed_desired.get(),
            SpeedControlAlgorithm::Adaptive => self.sa_adaptive.compute(self.speed_desired.get()),
        };

        self.speed_target.set(speed);
    }

    pub fn speed_target(&self) -> Velocity {
        self.speed_target.get()
    }
}

/// Controls adaptive puller speed based on laser diameter feedback.
///
/// # Behaviour
/// - **Inner deadzone** (`accepted_difference`): if `|current − target| ≤
///   accepted_difference` (mm) the measurement is considered close enough to the
///   target and no adjustment is made.  The meter accumulator is reset so the
///   delay always restarts when re-entering this zone.
/// - **Outer boundary** (`lower` / `upper` tolerances from the laser): if the
///   diameter leaves the inner deadzone, meters are accumulated.  After
///   `adjustment_interval_meters` have elapsed the modulation is nudged by
///   ±`step_percent` in the direction that brings the diameter back toward
///   target, and the accumulator is reset.
/// - **Soft limit**: modulation is clamped so the output speed never deviates
///   more than `max_speed_change_percent` % from the base speed.
pub struct SpeedAlgorithmAdaptive {
    speed_delta_max: ConfigProperty<f64>,
    increase_per_step: ConfigProperty<f64>,
    tolerance_limit: ConfigProperty<Length>,
    adjustment_distance: ConfigProperty<Length>,

    // internal state
    modulation: Measurement<f64>,
    distance_since_adjustment: Measurement<Length>,
}

// public interface
impl SpeedAlgorithmAdaptive {
    pub fn new(ctx: &mut BuildContext) -> BuildResult<Self> {
        Ok(Self {
            speed_delta_max: ctx
                .config::<f64>("puller.speed_controller.adaptive.speed_delta_max")
                .build()?,

            increase_per_step: ctx
                .config::<f64>("puller.speed_controller.adaptive.increase_per_step")
                .build()?,

            tolerance_limit: ctx
                .config::<millimeter>("puller.speed_controller.adaptive.tolerance_limit")
                .build()?,

            adjustment_distance: ctx
                .config::<meter>("puller.speed_controller.adaptive.adjustment_distance")
                .build()?,

            modulation: ctx
                .measurement::<f64>("puller.speed_controller.adaptive.modulation")
                .build()?,

            distance_since_adjustment: ctx
                .measurement::<meter>("puller.speed_controller.adaptive.distance_since_adjustment")
                .build()?,
        })
    }

    pub fn update(&mut self, dt: Duration, speed_prev: Velocity, current: f64, target: f64) {
        // --- Inner deadzone (accepted_difference) ---
        // If the diameter is within ±accepted_difference of the target it is
        // acceptable.  Reset the accumulator so the delay always starts fresh.
        if (current - target).abs() <= self.tolerance_limit.get_as::<millimeter>() {
            self.distance_since_adjustment.set_as::<millimeter>(0.0);
            return;
        }

        // --- Accumulate metres ---
        let meters_added = speed_prev.abs().get::<meter_per_second>() * dt.as_secs_f64();

        let distance_since_adjustment =
            self.distance_since_adjustment.get() + Length::new::<meter>(meters_added);

        self.distance_since_adjustment
            .set(distance_since_adjustment);

        // --- Wait for the interval to elapse ---
        if self.distance_since_adjustment.get() < self.adjustment_distance.get() {
            return;
        }

        // --- Apply one step in the required direction ---
        // Diameter too large  → speed up the puller (positive modulation)
        // Diameter too small  → slow down the puller (negative modulation)
        let correction_sign: f64 = if current > target { 1.0 } else { -1.0 };
        let step = self.increase_per_step.get() * correction_sign;
        self.modulation
            .set((self.modulation.get() + step).clamp(-1.0, 1.0));
        self.distance_since_adjustment.set(Length::ZERO);
    }

    pub fn compute(&self, base_speed: Velocity) -> Velocity {
        let factor = 1.0 + self.modulation.get() * self.speed_delta_max.get();
        (base_speed * factor).max(Velocity::ZERO)
    }

    /// Reset modulation to zero so the algorithm starts fresh from the base speed.
    pub fn reset_modulation(&mut self) {
        self.modulation.set(0.0);
        self.distance_since_adjustment.set(Length::ZERO);
    }
}
