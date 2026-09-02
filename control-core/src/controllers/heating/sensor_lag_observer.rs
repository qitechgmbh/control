//! Undoing a temperature sensor's own time constant.
//!
//! A probe in a pocket is a first-order lag: `tau * y' = x - y`, where `x` is
//! the metal and `y` is what the terminal reports. Rearranged, `x = y + tau * y'`,
//! so an estimate of the reading's slope buys back the lag.
//!
//! The slope is *not* `(y - y_prev) / dt`. With a kHz control loop reading a
//! terminal that converts every few hundred ms in 0.1 °C steps, that quotient is
//! zero on almost every tick and `0.1 / 0.001 = 100 K/s` on the tick the reading
//! moves — which is why a plain `kd` term on these loops is pure quantisation
//! noise. The washout form below never divides by `dt`:
//!
//! ```text
//! y_f += (1 - exp(-dt / tau_f)) * (y - y_f)     // low-pass of the reading
//! y'   = (y - y_f) / tau_f                      // its exact derivative
//! ```
//!
//! On a ramp of `r` K/s, `y - y_f` settles at `r * tau_f` and the slope estimate
//! at exactly `r`, while one quantisation step moves it by only `0.1 / tau_f`.
//! The lag is traded down rather than removed — the estimate still trails by
//! about `tau_f` — so `tau_f` several times below `tau_sensor` is the useful
//! regime.

use std::time::Instant;

/// Estimates the temperature of the metal from a lagging probe's reading.
#[derive(Debug, Clone)]
pub struct SensorLagObserver {
    /// The probe's own time constant in seconds — how far it trails the metal.
    tau_sensor_s: f64,
    /// Smoothing applied before differentiating, in seconds. Sets both the
    /// residual lag and how much sensor quantisation reaches the estimate;
    /// several times below `tau_sensor_s` is the useful regime.
    tau_filter_s: f64,
    /// Cap on the correction, in K. Bounds what a wrong `tau_sensor_s`, a
    /// sensor glitch or a step change of setpoint can do to the estimate.
    lead_max_k: f64,

    filtered_c: Option<f64>,
    last: Option<Instant>,
    rate_c_per_s: f64,
}

impl SensorLagObserver {
    pub const fn new(tau_sensor_s: f64, tau_filter_s: f64, lead_max_k: f64) -> Self {
        debug_assert!(
            tau_filter_s > 0.0,
            "tau_filter_s divides the slope estimate"
        );
        debug_assert!(tau_sensor_s >= 0.0);
        Self {
            tau_sensor_s,
            tau_filter_s,
            lead_max_k,
            filtered_c: None,
            last: None,
            rate_c_per_s: 0.0,
        }
    }

    /// Estimated metal temperature in °C, given this tick's reading.
    ///
    /// The first call has no history to differentiate and returns the reading
    /// unchanged.
    pub fn update(&mut self, measured_c: f64, now: Instant) -> f64 {
        let (Some(filtered), Some(last)) = (self.filtered_c, self.last) else {
            self.filtered_c = Some(measured_c);
            self.last = Some(now);
            self.rate_c_per_s = 0.0;
            return measured_c;
        };

        let dt = now.duration_since(last).as_secs_f64();
        self.last = Some(now);
        if dt <= 0.0 {
            // The caller can tick faster than the clock resolves. Re-use the
            // slope rather than dividing by zero.
            return self.estimate(measured_c);
        }

        // Exact discretisation of the low-pass, so an unexpectedly long `dt` —
        // a stall, a debugger, a slow cycle — cannot overshoot and ring the way
        // the `dt / tau` approximation does.
        let alpha = 1.0 - (-dt / self.tau_filter_s).exp();
        let filtered = alpha.mul_add(measured_c - filtered, filtered);
        self.filtered_c = Some(filtered);
        self.rate_c_per_s = (measured_c - filtered) / self.tau_filter_s;

        self.estimate(measured_c)
    }

    fn estimate(&self, measured_c: f64) -> f64 {
        let lead = (self.tau_sensor_s * self.rate_c_per_s).clamp(-self.lead_max_k, self.lead_max_k);
        measured_c + lead
    }

    pub const fn reset(&mut self) {
        self.filtered_c = None;
        self.last = None;
        self.rate_c_per_s = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct Run {
        /// Worst `|metal - reading|` after the skip window.
        raw_error_k: f64,
        /// Worst `|metal - estimate|` after the skip window.
        est_error_k: f64,
    }

    /// Drive a simulated first-order probe with a known metal temperature and
    /// report how far the raw reading and the estimate stray from the metal.
    ///
    /// `quantise` rounds the reading to 0.1 °C the way an EL3204 does. Errors
    /// before `skip_s` are ignored, so the observer's own start-up transient
    /// does not mask what is being measured.
    fn run(
        tau_sensor: f64,
        tau_filter: f64,
        metal_at: impl Fn(f64) -> f64,
        duration_s: f64,
        skip_s: f64,
        quantise: bool,
    ) -> Run {
        let dt = 0.01;
        let t0 = Instant::now();
        let mut obs = SensorLagObserver::new(tau_sensor, tau_filter, 200.0);
        let mut probe = metal_at(0.0);
        let mut out = Run {
            raw_error_k: 0.0,
            est_error_k: 0.0,
        };

        let steps = (duration_s / dt) as u64;
        for i in 0..steps {
            let t = i as f64 * dt;
            let metal = metal_at(t);
            probe = (dt / tau_sensor).mul_add(metal - probe, probe);
            let reading = if quantise {
                (probe * 10.0).round() / 10.0
            } else {
                probe
            };
            let estimate = obs.update(reading, t0 + Duration::from_nanos((t * 1e9) as u64));
            if t >= skip_s {
                out.raw_error_k = out.raw_error_k.max((metal - reading).abs());
                out.est_error_k = out.est_error_k.max((metal - estimate).abs());
            }
        }
        out
    }

    /// The point of the whole file: on a ramp the raw reading trails the metal
    /// by `tau_sensor * rate`, and the estimate must recover most of that.
    #[test]
    fn recovers_the_lag_on_a_ramp() {
        let rate: f64 = 0.23; // K/s, the extruder's cold-start ramp
        let r = run(150.0, 15.0, |t| rate.mul_add(t, 22.0), 1500.0, 750.0, false);

        assert!(
            (150.0f64.mul_add(-rate, r.raw_error_k)).abs() < 1.0,
            "raw reading should trail by tau*rate = {:.1} K, trailed {:.1} K",
            150.0 * rate,
            r.raw_error_k
        );
        assert!(
            r.est_error_k < 0.1 * r.raw_error_k,
            "estimate should cut the {:.1} K lag by 10x, left {:.1} K",
            r.raw_error_k,
            r.est_error_k
        );
    }

    /// The lag is *traded down*, not removed: after the ramp rate changes the
    /// estimate needs a few filter constants to catch up, not a few sensor
    /// constants. That difference is the whole benefit.
    #[test]
    fn converges_in_filter_constants_not_sensor_constants() {
        let rate: f64 = 0.23;
        let ramp = |t: f64| rate.mul_add(t, 22.0);
        // Five filter constants in — but only half a sensor constant, so the
        // raw reading is still nowhere near its steady trailing error.
        let early = run(150.0, 15.0, ramp, 75.0, 74.0, false);
        assert!(
            early.est_error_k < 3.0,
            "estimate still {:.2} K out after 5 filter constants",
            early.est_error_k
        );
        assert!(
            early.raw_error_k > 5.0 * early.est_error_k,
            "raw error {:.2} K should still dwarf the estimate's {:.2} K",
            early.raw_error_k,
            early.est_error_k
        );
    }

    /// Sensor quantisation must not be amplified into the estimate. Contrast
    /// with a `kd` term on the raw reading, which produces a full-scale kick
    /// from a single LSB.
    ///
    /// A slow ramp is the demanding case: the reading sits still, then steps a
    /// whole LSB at once.
    #[test]
    fn quantisation_does_not_blow_up_the_estimate() {
        // 0.02 K/s steps the reading by one LSB every 5 s.
        let r = run(
            150.0,
            15.0,
            |t| 0.02f64.mul_add(t, 180.0),
            1500.0,
            750.0,
            true,
        );
        assert!(
            r.est_error_k < 1.0,
            "quantisation drove the estimate {:.2} K off a 0.02 K/s ramp",
            r.est_error_k
        );
    }

    /// The cap has to hold even when `tau_sensor_s` is badly wrong, because on
    /// the real machine it is only known to within a factor of a few.
    #[test]
    fn the_lead_correction_is_capped() {
        let t0 = Instant::now();
        let mut obs = SensorLagObserver::new(150.0, 15.0, 5.0);
        let mut lead = 0.0;
        for i in 0..100_000u64 {
            // 1 K/s: four times the real ramp, so an uncapped lead would be
            // 150 K.
            let t = i as f64 * 0.01;
            lead = obs.update(t, t0 + Duration::from_nanos((t * 1e9) as u64)) - t;
            assert!(lead <= 5.0 + 1e-9, "lead {lead} exceeded the 5 K cap");
        }
        assert!(
            (lead - 5.0).abs() < 1e-9,
            "cap should be active on a 1 K/s ramp, lead was only {lead}"
        );
    }

    #[test]
    fn the_first_reading_passes_through_unchanged() {
        let mut obs = SensorLagObserver::new(150.0, 15.0, 40.0);
        assert!((obs.update(22.0, Instant::now()) - 22.0).abs() < f64::EPSILON);
    }

    /// A long `dt` must settle towards the reading, never past it. The
    /// `dt / tau` approximation goes unstable here; the exponential does not.
    #[test]
    fn a_huge_time_step_stays_bounded() {
        let t0 = Instant::now();
        let mut obs = SensorLagObserver::new(150.0, 15.0, 1e9);
        obs.update(0.0, t0);
        // 100 s in one jump, against a 15 s filter.
        let estimate = obs.update(100.0, t0 + Duration::from_secs(100));
        assert!(
            (100.0..=102.0).contains(&estimate),
            "estimate {estimate} left the sane range after a 100 s step"
        );
    }
}
