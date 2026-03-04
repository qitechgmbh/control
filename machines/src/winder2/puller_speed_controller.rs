use std::time::Instant;

use control_core::{
    controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    converters::linear_step_converter::LinearStepConverter,
};
use serde::{Deserialize, Serialize};
use units::ConstZero;
use units::acceleration::meter_per_minute_per_second;
use units::f64::*;
use units::jerk::meter_per_minute_per_second_squared;
use units::velocity::{meter_per_minute, meter_per_second};

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

        let adaptive = AdaptiveSpeedAlgorithm {
            speed_base: target_speed,
            max_speed_change_percent: 5.0,
            adjustment_interval_meters: 20.0,
            step_percent: 1.0,
            accepted_difference: 0.03,
            modulation: 0.0,
            meters_since_last_adjustment: 0.0,
            last_update: Instant::now(),
        };

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

    pub const fn set_regulation_mode(&mut self, regulation: PullerRegulationMode) {
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
                PullerRegulationMode::Diameter => self.adaptive.compute(),
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
    // --- Configurable (Control Page) ---
    /// Base puller speed; all modulation is relative to this.
    pub speed_base: Velocity,

    // --- Configurable (Settings Page) ---
    /// Maximum allowed speed change as a percentage of base speed (0.0–100.0).
    pub max_speed_change_percent: f64,
    /// Minimum meters the puller must travel between consecutive adjustments.
    pub adjustment_interval_meters: f64,
    /// Step size per adjustment as a percentage of base speed (0.0–100.0).
    pub step_percent: f64,
    /// Inner deadzone: maximum deviation from target (mm) that is considered
    /// acceptable.  No adjustment is made while `|current − target| ≤ this`.
    pub accepted_difference: f64,

    // --- Internal state (not exposed to the frontend) ---
    /// Current modulation in [-1.0, 1.0].
    /// Output = speed_base × (1 + modulation × max_speed_change_percent / 100)
    modulation: f64,
    /// Meters accumulated while outside the inner deadzone since the last adjustment.
    meters_since_last_adjustment: f64,
    /// Timestamp of the previous `update_with_measurement` call for computing Δt.
    last_update: Instant,
}

impl AdaptiveSpeedAlgorithm {
    /// Compute the current target speed after applying the active modulation.
    pub fn compute(&self) -> Velocity {
        let factor = 1.0 + self.modulation * self.max_speed_change_percent / 100.0;
        (self.speed_base * factor).max(Velocity::ZERO)
    }

    pub fn speed_base(&self) -> Velocity {
        self.speed_base
    }

    pub fn set_speed_base(&mut self, speed: Velocity) {
        self.speed_base = speed.max(Velocity::ZERO);
    }

    /// Current modulation level in [-1.0, 1.0].
    pub fn modulation(&self) -> f64 {
        self.modulation
    }

    pub fn max_speed_change_percent(&self) -> f64 {
        self.max_speed_change_percent
    }

    pub fn set_max_speed_change_percent(&mut self, percent: f64) {
        self.max_speed_change_percent = percent.max(0.0);
    }

    pub fn adjustment_interval_meters(&self) -> f64 {
        self.adjustment_interval_meters
    }

    pub fn set_adjustment_interval_meters(&mut self, meters: f64) {
        self.adjustment_interval_meters = meters.max(0.0);
    }

    pub fn step_percent(&self) -> f64 {
        self.step_percent
    }

    pub fn set_step_percent(&mut self, percent: f64) {
        self.step_percent = percent.clamp(0.0, 100.0);
    }

    pub fn accepted_difference(&self) -> f64 {
        self.accepted_difference
    }

    pub fn set_accepted_difference(&mut self, mm: f64) {
        self.accepted_difference = mm.max(0.0);
    }

    /// Process a new laser diameter measurement.
    ///
    /// `current`    — measured diameter (mm)
    /// `target`     — target diameter (mm)
    /// `lower`      — lower tolerance relative to target (mm, positive value)
    /// `upper`      — upper tolerance relative to target (mm, positive value)
    /// `last_speed` — puller speed at the previous cycle, used to convert Δt → metres
    /// `now`        — current instant (enables deterministic testing)
    pub fn update_with_measurement(
        &mut self,
        current: f64,
        target: f64,
        lower: f64,
        upper: f64,
        last_speed: Velocity,
        now: Instant,
    ) {
        let dt = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;

        let lower_bound = target - lower;
        let upper_bound = target + upper;
        let _ = (lower_bound, upper_bound); // kept for future use (e.g. trend detection)

        // ── Inner deadzone (accepted_difference) ────────────────────────────────
        // If the diameter is within ±accepted_difference of the target it is
        // acceptable.  Reset the accumulator so the delay always starts fresh.
        if (current - target).abs() <= self.accepted_difference {
            self.meters_since_last_adjustment = 0.0;
            return;
        }

        // ── Accumulate metres ───────────────────────────────────────────────────
        let meters_added = last_speed.abs().get::<meter_per_second>() * dt;
        self.meters_since_last_adjustment += meters_added;

        // ── Wait for the interval to elapse ─────────────────────────────────────
        if self.meters_since_last_adjustment < self.adjustment_interval_meters {
            return;
        }

        // ── Apply one step in the required direction ─────────────────────────────
        // Diameter too large  → speed up the puller (positive modulation)
        // Diameter too small  → slow down the puller (negative modulation)
        let direction = if current > target { 1.0_f64 } else { -1.0_f64 };
        let step = self.step_percent / 100.0;
        self.modulation = (self.modulation + direction * step).clamp(-1.0, 1.0);
        self.meters_since_last_adjustment = 0.0;
    }
}
