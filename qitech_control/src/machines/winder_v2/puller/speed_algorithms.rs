use std::time::Instant;

use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Length;
use qitech_lib::units::Velocity;
use qitech_lib::units::length::meter;
use qitech_lib::units::length::millimeter;
use qitech_lib::units::velocity::meter_per_second;

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
    pub speed_delta_max: ConfigProperty<f64>,
    pub increase_per_step: ConfigProperty<f64>,
    pub tolerance_limit: ConfigProperty<Length>,
    pub adjustment_distance: ConfigProperty<Length>,

    // internal state
    pub modulation: Measurement<f64>,
    pub distance_since_adjustment: Measurement<Length>,
    pub time_since_last_update: Instant,
}

// public interface
impl SpeedAlgorithmAdaptive {
    pub fn update(
        &mut self,
        now: Instant,
        speed: Velocity,
        current: f64,
        target: f64,
        lower: f64,
        upper: f64,
    ) {
        let dt = now
            .duration_since(self.time_since_last_update)
            .as_secs_f64();
        self.time_since_last_update = now;

        let lower_bound = target - lower;
        let upper_bound = target + upper;
        let _ = (lower_bound, upper_bound); // kept for future use (e.g. trend detection)

        // --- Inner deadzone (accepted_difference) ---
        // If the diameter is within ±accepted_difference of the target it is
        // acceptable.  Reset the accumulator so the delay always starts fresh.
        if (current - target).abs() <= self.tolerance_limit.get_as::<millimeter>() {
            self.distance_since_adjustment.set_as::<millimeter>(0.0);
            return;
        }

        // --- Accumulate metres ---
        let meters_added = speed.abs().get::<meter_per_second>() * dt;

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
