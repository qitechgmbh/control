use std::time::Instant;

use control_core::{
    controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    converters::linear_step_converter::LinearStepConverter,
};
use serde::{Deserialize, Serialize};
use qitech_lib::units::ConstZero;
use qitech_lib::units::acceleration::meter_per_minute_per_second;
use qitech_lib::units::f64::Length;
use qitech_lib::units::f64::*;
use qitech_lib::units::jerk::meter_per_minute_per_second_squared;
use qitech_lib::units::length::{meter, millimeter};
use qitech_lib::units::velocity::{meter_per_minute, meter_per_second};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum GearRatio {
    OneToOne,
    OneToFive,
    OneToTen,
}

impl GearRatio {
    /// Get the speed multiplier for this gear ratio
    pub fn multiplier(&self) -> f64 {
        match self {
            GearRatio::OneToOne => 1.0,
            GearRatio::OneToFive => 5.0,
            GearRatio::OneToTen => 10.0,
        }
    }
}

impl Default for GearRatio {
    fn default() -> Self {
        GearRatio::OneToOne
    }
}

#[derive(Debug)]
pub struct PullerSpeedController {
    enabled: bool,
    pub target_speed: Velocity,

    pub adaptive: AdaptiveSpeedAlgorithm,

    pub regulation_mode: PullerRegulationMode,
    /// Forward rotation direction. If false, applies negative sign to speed
    pub forward: bool,
    /// Gear ratio for winding speed (1:5 or 1:10)
    pub gear_ratio: GearRatio,
    /// Linear acceleration controller to dampen speed change
    acceleration_controller: LinearJerkSpeedController,
    /// Converter for linear to angular transformations
    pub converter: LinearStepConverter,
    pub last_speed: Velocity,
}

impl PullerSpeedController {
    pub fn new(target_speed: Velocity, converter: LinearStepConverter) -> Self {
        let acceleration = Acceleration::new::<meter_per_minute_per_second>(5.0);
        let jerk = Jerk::new::<meter_per_minute_per_second_squared>(10.0);
        let speed = Velocity::new::<meter_per_minute>(50.0);

        let mut adaptive = AdaptiveSpeedAlgorithm::default();
        adaptive.set_speed_delta_max(0.33);
        adaptive.set_increase_per_step(0.033);
        adaptive.set_tolerance_limit(Length::new::<millimeter>(0.01));
        adaptive.set_adjustment_distance(Length::new::<meter>(0.5));

        Self {
            enabled: false,
            target_speed,
            adaptive,
            regulation_mode: PullerRegulationMode::Speed,
            forward: true,
            gear_ratio: GearRatio::default(),
            acceleration_controller: LinearJerkSpeedController::new_simple(
                Some(speed),
                acceleration,
                jerk,
            ),
            converter,
            last_speed: Velocity::ZERO,
        }
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_target_speed(&mut self, target: Velocity) {
        self.target_speed = target;
    }

    pub fn set_regulation_mode(&mut self, regulation: PullerRegulationMode) {
        // Reset adaptive modulation when switching to Diameter mode
        // so it starts from the current target_speed without jumps
        if matches!(regulation, PullerRegulationMode::Diameter) {
            self.adaptive.reset_modulation();
        }
        self.regulation_mode = regulation;
    }

    pub const fn set_forward(&mut self, forward: bool) {
        self.forward = forward;
    }

    pub const fn set_gear_ratio(&mut self, gear_ratio: GearRatio) {
        self.gear_ratio = gear_ratio;
    }

    pub const fn get_gear_ratio(&self) -> GearRatio {
        self.gear_ratio
    }

    fn update_speed(&mut self, t: Instant) -> Velocity {
        let base_speed = match self.enabled {
            true => match self.regulation_mode {
                PullerRegulationMode::Speed => self.target_speed,
                PullerRegulationMode::Diameter => self.adaptive.compute(self.target_speed),
            },
            false => Velocity::ZERO,
        };

        // Apply gear ratio multiplier
        let speed = base_speed * self.gear_ratio.multiplier();

        let speed = if self.forward { speed } else { -speed };

        let speed = self.acceleration_controller.update(speed, t);

        self.last_speed = speed;
        speed
    }

    pub fn speed_to_angular_velocity(&self, speed: Velocity) -> AngularVelocity {
        // Use the converter to transform from linear velocity to angular velocity
        self.converter.velocity_to_angular_velocity(speed)
    }

    pub fn angular_velocity_to_speed(&self, angular_speed: AngularVelocity) -> Velocity {
        // Use the converter to transform from angular velocity to linear velocity
        self.converter.angular_velocity_to_velocity(angular_speed)
    }

    pub fn calc_angular_velocity(&mut self, t: Instant) -> AngularVelocity {
        let speed = self.update_speed(t);
        self.speed_to_angular_velocity(speed)
    }

    pub fn get_target_speed(&self) -> Velocity {
        self.target_speed
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum PullerRegulationMode {
    #[default]
    Speed,
    Diameter,
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
#[derive(Debug, Clone)]
pub struct AdaptiveSpeedAlgorithm {
    // config
    speed_delta_max: f64,
    increase_per_step: f64,
    tolerance_limit: Length,
    adjustment_distance: Length,

    // internal state
    modulation: f64,
    distance_since_last_adjustment: Length,
    time_since_last_update: Instant,
}

impl Default for AdaptiveSpeedAlgorithm {
    fn default() -> Self {
        Self {
            speed_delta_max: 0.0,
            increase_per_step: 0.0,
            adjustment_distance: Length::ZERO,
            tolerance_limit: Length::ZERO,
            modulation: 0.0,
            distance_since_last_adjustment: Length::ZERO,
            time_since_last_update: Instant::now(),
        }
    }
}

// public interface
impl AdaptiveSpeedAlgorithm {
    pub fn compute(&self, base_speed: Velocity) -> Velocity {
        let factor = 1.0 + self.modulation * self.speed_delta_max;
        (base_speed * factor).max(Velocity::ZERO)
    }

    pub fn update_with_measurement(
        &mut self,
        current: f64,
        target: f64,
        lower: f64,
        upper: f64,
        last_speed: Velocity,
        now: Instant,
    ) {
        let dt = now
            .duration_since(self.time_since_last_update)
            .as_secs_f64();
        self.time_since_last_update = now;

        let lower_bound = target - lower;
        let upper_bound = target + upper;
        let _ = (lower_bound, upper_bound); // kept for future use (e.g. trend detection)

        // ── Inner deadzone (accepted_difference) ────────────────────────────────
        // If the diameter is within ±accepted_difference of the target it is
        // acceptable.  Reset the accumulator so the delay always starts fresh.
        if (current - target).abs() <= self.tolerance_limit.get::<millimeter>() {
            self.distance_since_last_adjustment = Length::ZERO;
            return;
        }

        // ── Accumulate metres ───────────────────────────────────────────────────
        let meters_added = last_speed.abs().get::<meter_per_second>() * dt;
        self.distance_since_last_adjustment += Length::new::<meter>(meters_added);

        // ── Wait for the interval to elapse ─────────────────────────────────────
        if self.distance_since_last_adjustment < self.adjustment_distance {
            return;
        }

        // ── Apply one step in the required direction ─────────────────────────────
        // Diameter too large  → speed up the puller (positive modulation)
        // Diameter too small  → slow down the puller (negative modulation)
        let correction_sign: f64 = if current > target { 1.0 } else { -1.0 };
        let step = self.increase_per_step * correction_sign;
        self.modulation = (self.modulation + step).clamp(-1.0, 1.0);
        self.distance_since_last_adjustment = Length::ZERO;
    }
}

// getters + setters
impl AdaptiveSpeedAlgorithm {
    pub fn speed_delta_max(&self) -> f64 {
        self.speed_delta_max
    }

    pub fn set_speed_delta_max(&mut self, value: f64) {
        self.speed_delta_max = value.max(0.0);
    }

    pub fn increase_per_step(&self) -> f64 {
        self.increase_per_step
    }

    pub fn set_increase_per_step(&mut self, value: f64) {
        self.increase_per_step = value.max(0.0).min(1.0);
    }

    pub fn adjustment_distance(&self) -> Length {
        self.adjustment_distance
    }

    pub fn set_adjustment_distance(&mut self, value: Length) {
        self.adjustment_distance = value.max(Length::ZERO);
    }

    pub fn tolerance_limit(&self) -> Length {
        self.tolerance_limit
    }

    pub fn set_tolerance_limit(&mut self, value: Length) {
        self.tolerance_limit = value.max(Length::ZERO);
    }

    /// Current modulation level in [-1.0, 1.0].
    pub fn modulation(&self) -> f64 {
        self.modulation
    }

    /// Reset modulation to zero so the algorithm starts fresh from the base speed.
    pub fn reset_modulation(&mut self) {
        self.modulation = 0.0;
        self.distance_since_last_adjustment = Length::ZERO;
    }
}
