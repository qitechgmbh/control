use std::time::Instant;

/// Default band around the setpoint, in error units, inside which the integral accumulates.
/// Outside it the accumulator is frozen (not reset), so a large approach does not wind it up.
const DEFAULT_INTEGRATION_BAND: f64 = 15.0;

/// Time constant of the filter on the measurement rate used for stall detection, in seconds.
/// Deliberately slow — this signal answers "is the temperature still moving", not "what is the
/// derivative right now", so lag is fine and noise immunity is everything.
const STALL_RATE_FILTER_TC: f64 = 5.0;

#[derive(Debug)]
pub struct PidController {
    // Params
    /// Proportional gain
    kp: f64,
    /// Integral gain
    ki: f64,
    /// Derivative gain
    kd: f64,
    // State
    /// Proportional error
    ep: f64,
    /// Integral error
    ei: f64,
    /// Derivative error (post-filter, when filtering is enabled)
    ed: f64,

    i_min: f64,
    i_max: f64,

    /// Time constant of the first-order filter on the derivative term, in seconds.
    ///
    /// `0.0` disables filtering, reproducing a plain backward difference. An unfiltered derivative
    /// over a sub-millisecond loop period is a noise amplifier: one sensor LSB of flicker becomes a
    /// large transient, because `dt` sits in the denominator. Set this to roughly `Td / 10` whenever
    /// `kd` is non-zero.
    derivative_filter_tc: f64,
    /// Previous measurement, for derivative-on-measurement. `None` until the first such update.
    last_measurement: Option<f64>,

    /// Measurement rate below which the process counts as stalled, in units per second.
    /// `0.0` disables stall-aware integration, preserving the plain band-freeze behaviour.
    ///
    /// The band freeze alone has a failure mode with small gains: if the proportional term cannot
    /// carry the standing load by itself, the process settles *outside* the band, where the
    /// integral is frozen — so it can never accumulate the load, and the offset is permanent.
    /// With a stall threshold set, a process that has stopped moving is allowed to integrate even
    /// outside the band; as soon as it moves again the freeze returns, so approach windup stays
    /// suppressed. The result is a ratchet that walks the output up to the standing load.
    stall_rate_threshold: f64,
    /// Filtered measurement rate, units per second, used for stall detection.
    stall_rate: f64,

    last: Option<Instant>,
}
impl PidController {
    pub const fn new(kp: f64, ki: f64, kd: f64, i_min: f64, i_max: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            ep: 0.0,
            ei: 0.0,
            ed: 0.0,
            last: None,
            i_min,
            i_max,
            derivative_filter_tc: 0.0,
            last_measurement: None,
            stall_rate_threshold: 0.0,
            stall_rate: 0.0,
        }
    }

    /// Enable stall-aware integration: when the filtered measurement rate is below `threshold`
    /// (units per second), the integral may accumulate even outside the integration band. Pass
    /// `0.0` to disable and restore the plain band-freeze behaviour.
    pub const fn set_stall_integration(&mut self, threshold: f64) {
        self.stall_rate_threshold = if threshold > 0.0 { threshold } else { 0.0 };
    }

    /// Seed the integral so that its contribution to the output equals `contribution`.
    ///
    /// Used when tuned gains are applied while the process is at its operating point: the standing
    /// output that was holding it there is known, and starting the integral from zero would drop
    /// the output to the (small) proportional term alone — collapsing the process away from its
    /// setpoint and, with a small integral gain, taking tens of minutes to recover.
    ///
    /// The contribution is clamped to the integral limits. A no-op while `ki` is zero.
    pub fn preload_integral(&mut self, contribution: f64) {
        if self.ki != 0.0 {
            self.ei = contribution.clamp(self.i_min, self.i_max) / self.ki;
        }
    }

    pub const fn configure(&mut self, ki: f64, kp: f64, kd: f64) {
        self.reset();
        self.kp = kp;
        self.ki = ki;
        self.kd = kd;
    }

    /// Set the derivative filter time constant in seconds. `0.0` disables filtering.
    ///
    /// Does not reset the controller — the filter can be retuned without disturbing the integral.
    pub const fn set_derivative_filter(&mut self, tc: f64) {
        self.derivative_filter_tc = if tc > 0.0 { tc } else { 0.0 };
    }

    pub const fn get_derivative_filter(&self) -> f64 {
        self.derivative_filter_tc
    }

    pub const fn get_kp(&self) -> f64 {
        self.kp
    }

    pub const fn get_ki(&self) -> f64 {
        self.ki
    }

    pub const fn get_kd(&self) -> f64 {
        self.kd
    }

    /// Update with the derivative taken on the error signal.
    ///
    /// A setpoint change steps the error, so the derivative term spikes ("derivative kick"). Prefer
    /// [`Self::update_with_measurement`] wherever the measurement is available.
    pub fn update(&mut self, error: f64, t: Instant) -> f64 {
        self.update_inner(error, None, t)
    }

    /// Update with the derivative taken on the measurement rather than the error.
    ///
    /// Since `error = setpoint - measurement`, the two agree exactly while the setpoint is constant.
    /// They differ only when the setpoint moves, where this form avoids the derivative kick.
    pub fn update_with_measurement(&mut self, error: f64, measurement: f64, t: Instant) -> f64 {
        self.update_inner(error, Some(measurement), t)
    }

    fn update_inner(&mut self, error: f64, measurement: Option<f64>, t: Instant) -> f64 {
        match self.last {
            // First update
            None => {
                // Calculate error
                let ep = error;

                // Calculate signal. The integral is kept, not zeroed: `reset()` already cleared it
                // unless it was deliberately seeded via `preload_integral`, in which case dropping
                // it here would collapse the output for exactly the case the preload exists for.
                let signal = (self.ki * self.ei).clamp(self.i_min, self.i_max) + self.kp * ep;

                // Set values
                self.ep = ep;
                self.ed = 0.0;
                self.last_measurement = measurement;
                self.last = Some(t);

                signal
            }
            // Subsequent updates
            Some(last) => {
                // Calculate the time delta in seconds
                let dt = t.duration_since(last).as_secs_f64();

                // No time has passed: there is nothing to integrate or differentiate, and dividing
                // by dt would poison the controller state with NaN permanently.
                if dt <= 0.0 {
                    return (self.ki * self.ei).clamp(self.i_min, self.i_max)
                        + self.kd.mul_add(self.ed, self.kp * error);
                }

                // Calculate errors
                let ep = error;

                // Measurement rate for stall detection: d(pv)/dt when the measurement is
                // available, -d(error)/dt otherwise (identical for a constant setpoint). Filtered
                // slowly — it answers "is the process still moving", nothing faster.
                let rate_raw = match (measurement, self.last_measurement) {
                    (Some(pv), Some(prev)) => (pv - prev) / dt,
                    _ => -(ep - self.ep) / dt,
                };
                let rate_alpha = dt / (STALL_RATE_FILTER_TC + dt);
                self.stall_rate = rate_alpha.mul_add(rate_raw - self.stall_rate, self.stall_rate);

                // Integrate inside the band, or — when stall-aware integration is enabled —
                // whenever the process has stopped moving. A stalled process outside the band is
                // one the proportional term alone cannot bring home; without this the integral
                // stays frozen out there and the offset is permanent. The moment the process moves
                // again the freeze returns, so approach windup stays suppressed.
                let stalled = self.stall_rate_threshold > 0.0
                    && self.stall_rate.abs() < self.stall_rate_threshold;
                let ei = if error.abs() < DEFAULT_INTEGRATION_BAND || stalled {
                    ep.mul_add(dt, self.ei)
                } else {
                    self.ei
                };

                // Derivative on measurement when it is available, on error otherwise.
                let ed_raw = match (measurement, self.last_measurement) {
                    (Some(pv), Some(prev)) => -(pv - prev) / dt,
                    _ => (ep - self.ep) / dt,
                };

                // First-order filter. tc == 0.0 passes the raw derivative through unchanged.
                let ed = if self.derivative_filter_tc > 0.0 {
                    let alpha = dt / (self.derivative_filter_tc + dt);
                    alpha.mul_add(ed_raw - self.ed, self.ed)
                } else {
                    ed_raw
                };

                let kp_signal = self.kp * ep;
                let ki_signal = (self.ki * ei).clamp(self.i_min, self.i_max);
                let kd_signal = self.kd * ed;

                // Calculate signal
                let signal = kp_signal + ki_signal + kd_signal;

                // Set values
                self.ep = ep;
                // Back-calculate the accumulator from the clamped contribution: this is the
                // anti-windup, and it is what makes a wide i_min/i_max safe.
                self.ei = if self.ki != 0.0 {
                    ki_signal / self.ki
                } else {
                    0.0
                };
                self.ed = ed;
                self.last_measurement = measurement;
                self.last = Some(t);

                signal
            }
        }
    }

    pub const fn reset(&mut self) {
        self.ep = 0.0;
        self.ei = 0.0;
        self.ed = 0.0;
        self.stall_rate = 0.0;
        self.last = None;
        self.last_measurement = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// With the filter disabled the derivative is a plain backward difference, unchanged from the
    /// behaviour every existing caller relies on.
    #[test]
    fn unfiltered_derivative_is_a_plain_backward_difference() {
        let mut pid = PidController::new(0.0, 0.0, 1.0, -1.0, 1.0);
        let t0 = Instant::now();
        let dt = Duration::from_millis(100);

        assert_eq!(pid.update(0.0, t0), 0.0);
        // kp = ki = 0, so the signal is purely kd * (ep - ep_prev) / dt.
        let signal = pid.update(2.0, t0 + dt);
        assert!((signal - (2.0 - 0.0) / 0.1).abs() < 1e-9);
        assert_eq!(pid.get_derivative_filter(), 0.0);
    }

    /// The filter must bound the response to a single-sample blip. Unfiltered, a small step over a
    /// sub-millisecond dt produces an enormous derivative; filtered, it is attenuated by roughly
    /// dt / (tc + dt).
    #[test]
    fn derivative_filter_attenuates_a_single_sample_blip() {
        let dt = Duration::from_micros(300);
        let blip = 0.008; // one LSB after the upstream low-pass, in °C

        let mut unfiltered = PidController::new(0.0, 0.0, 0.008, -1.0, 1.0);
        let t0 = Instant::now();
        unfiltered.update(0.0, t0);
        let raw = unfiltered.update(blip, t0 + dt);

        let mut filtered = PidController::new(0.0, 0.0, 0.008, -1.0, 1.0);
        filtered.set_derivative_filter(1.0); // Td/N with Td ~= 10 s, N = 10
        filtered.update(0.0, t0);
        let smoothed = filtered.update(blip, t0 + dt);

        // Unfiltered this is a large fraction of full duty; filtered it is negligible.
        assert!(raw > 0.15, "expected a large unfiltered spike, got {raw}");
        assert!(
            smoothed < raw / 1000.0,
            "filter should attenuate by ~dt/(tc+dt); raw={raw} smoothed={smoothed}"
        );
    }

    /// While the setpoint is constant, derivative-on-measurement and derivative-on-error agree.
    #[test]
    fn derivative_forms_agree_while_setpoint_is_constant() {
        let setpoint = 100.0;
        let t0 = Instant::now();
        let dt = Duration::from_millis(100);

        let mut on_error = PidController::new(0.5, 0.0, 2.0, -1.0, 1.0);
        let mut on_measurement = PidController::new(0.5, 0.0, 2.0, -1.0, 1.0);

        let mut a = 0.0;
        let mut b = 0.0;
        for (i, pv) in [90.0_f64, 92.0, 95.0, 97.0].into_iter().enumerate() {
            let t = t0 + dt * (i as u32);
            a = on_error.update(setpoint - pv, t);
            b = on_measurement.update_with_measurement(setpoint - pv, pv, t);
        }
        assert!((a - b).abs() < 1e-9, "{a} vs {b}");
    }

    /// A setpoint change kicks the error derivative but not the measurement derivative.
    #[test]
    fn derivative_on_measurement_has_no_setpoint_kick() {
        let t0 = Instant::now();
        let dt = Duration::from_millis(100);
        let pv = 90.0;

        let mut on_error = PidController::new(0.0, 0.0, 1.0, -100.0, 100.0);
        let mut on_measurement = PidController::new(0.0, 0.0, 1.0, -100.0, 100.0);

        on_error.update(100.0 - pv, t0);
        on_measurement.update_with_measurement(100.0 - pv, pv, t0);

        // Setpoint jumps 100 -> 150 while the measurement is unchanged.
        let kicked = on_error.update(150.0 - pv, t0 + dt);
        let calm = on_measurement.update_with_measurement(150.0 - pv, pv, t0 + dt);

        assert!(
            (kicked - 500.0).abs() < 1e-9,
            "expected a kick, got {kicked}"
        );
        assert_eq!(calm, 0.0, "measurement did not move, derivative must be 0");
    }

    /// The widened clamp must let the integral carry more than the old 20% cap, which is what an
    /// IMC-tuned ki depends on to remove steady-state offset.
    #[test]
    fn integral_can_exceed_the_old_twenty_percent_cap() {
        let mut pid = PidController::new(0.0, 0.01, 0.0, -1.0, 1.0);
        let t0 = Instant::now();
        let dt = Duration::from_millis(100);

        let mut signal = 0.0;
        for i in 0..2000 {
            // Error inside the conditional-integration band so the accumulator runs.
            signal = pid.update(1.0, t0 + dt * i);
        }
        assert!(
            signal > 0.2,
            "integral should reach past the old i_max of 0.2, got {signal}"
        );
        assert!(signal <= 1.0, "must still respect i_max, got {signal}");
    }

    /// The integral must be able to go negative to unwind an overshoot — impossible under the old
    /// i_min = 0.0.
    #[test]
    fn integral_can_go_negative() {
        let mut pid = PidController::new(0.0, 0.01, 0.0, -1.0, 1.0);
        let t0 = Instant::now();
        let dt = Duration::from_millis(100);

        let mut signal = 0.0;
        for i in 0..500 {
            signal = pid.update(-1.0, t0 + dt * i);
        }
        assert!(signal < 0.0, "expected a negative integral, got {signal}");
    }

    /// Anti-windup: the back-calculation must hold the accumulator at the clamp, so the controller
    /// recovers as soon as the error reverses instead of unwinding for a long time first.
    #[test]
    fn integral_does_not_wind_up_past_the_clamp() {
        let mut pid = PidController::new(0.0, 0.01, 0.0, -0.5, 0.5);
        let t0 = Instant::now();
        let dt = Duration::from_millis(100);

        // Drive hard into the clamp for a long time.
        for i in 0..5000 {
            pid.update(1.0, t0 + dt * i);
        }
        // One step of opposite error should immediately pull the output below the clamp.
        let after = pid.update(-1.0, t0 + dt * 5000);
        assert!(after < 0.5, "expected immediate unwind, got {after}");
    }

    /// A repeated timestamp must not produce NaN or infinity.
    #[test]
    fn zero_dt_is_safe() {
        let mut pid = PidController::new(1.0, 0.1, 1.0, -1.0, 1.0);
        let t = Instant::now();
        pid.update(1.0, t);
        let signal = pid.update(1.0, t);
        assert!(signal.is_finite(), "got {signal}");
        assert!(pid.update(1.0, t).is_finite());
    }

    /// Integration is frozen, not reset, outside the conditional band.
    #[test]
    fn integration_freezes_outside_the_band() {
        let mut pid = PidController::new(0.0, 0.01, 0.0, -1.0, 1.0);
        let t0 = Instant::now();
        let dt = Duration::from_millis(100);

        // Accumulate inside the band.
        for i in 0..100 {
            pid.update(1.0, t0 + dt * i);
        }
        let inside = pid.update(1.0, t0 + dt * 100);

        // Jump far outside the band; the accumulator should hold its value.
        let outside = pid.update(50.0, t0 + dt * 101);
        assert!(
            (outside - inside).abs() < 1e-3,
            "integral should freeze, not reset: {inside} -> {outside}"
        );
    }

    /// Applying tuned gains at the operating point: the preloaded integral must carry the standing
    /// output from the very first update, not collapse to the proportional term alone.
    #[test]
    fn preloaded_integral_survives_configure_and_first_update() {
        let mut pid = PidController::new(0.16, 0.0, 0.008, -0.95, 0.95);
        // Apply IMC-like gains (small kp, tiny ki), then seed with the measured standing load.
        pid.configure(0.00005, 0.01, 0.0);
        pid.preload_integral(0.37);

        let t0 = Instant::now();
        // First update, at setpoint: output should be the standing load, not ~zero.
        let first = pid.update_with_measurement(0.0, 178.5, t0);
        assert!(
            (first - 0.37).abs() < 1e-9,
            "expected the preloaded standing load, got {first}"
        );
        // And it must persist across subsequent updates.
        let second = pid.update_with_measurement(0.0, 178.5, t0 + Duration::from_millis(100));
        assert!((second - 0.37).abs() < 1e-6, "got {second}");
    }

    #[test]
    fn preload_respects_the_integral_clamp_and_zero_ki() {
        let mut pid = PidController::new(0.01, 0.00005, 0.0, -0.5, 0.5);
        pid.preload_integral(0.9); // beyond i_max
        let signal = pid.update(0.0, Instant::now());
        assert!((signal - 0.5).abs() < 1e-9, "got {signal}");

        let mut no_ki = PidController::new(0.01, 0.0, 0.0, -0.5, 0.5);
        no_ki.preload_integral(0.9); // must be a no-op, not a division by zero
        assert_eq!(no_ki.update(0.0, Instant::now()), 0.0);
    }

    /// The deadlock this exists for: with a small kp, the process stalls outside the ±5 °C band
    /// where the plain freeze keeps the integral at zero forever. With stall-aware integration the
    /// accumulator must grow once the measurement stops moving.
    #[test]
    fn stall_integration_unfreezes_a_stalled_process_outside_the_band() {
        let mut pid = PidController::new(0.01, 0.00005, 0.0, -0.95, 0.95);
        pid.set_stall_integration(0.005);

        let t0 = Instant::now();
        let dt = Duration::from_millis(100);
        // Process stalled 38 °C below setpoint: constant measurement, constant error.
        let mut signal = 0.0;
        for i in 0..3000 {
            signal = pid.update_with_measurement(38.0, 140.0, t0 + dt * i);
        }
        let p_only = 0.01 * 38.0;
        assert!(
            signal > p_only + 0.05,
            "integral should accumulate while stalled: signal={signal}, P alone={p_only}"
        );
    }

    /// While the process is actually moving toward setpoint, the freeze must stay in force — that
    /// is the approach-windup protection the band exists for.
    #[test]
    fn stall_integration_stays_frozen_while_the_process_moves() {
        let mut pid = PidController::new(0.01, 0.00005, 0.0, -0.95, 0.95);
        pid.set_stall_integration(0.005);

        let t0 = Instant::now();
        let dt = Duration::from_millis(100);
        // Heating at 3 °C/min — far above the stall threshold — from 100 °C below setpoint.
        // Skip the first few seconds: the rate filter needs a moment to see the movement.
        let mut integral_after_warmup = None;
        let mut last_signal = 0.0;
        for i in 0..6000_u32 {
            let pv = 78.5 + f64::from(i) * 0.1 * (3.0 / 60.0);
            let error = 178.5 - pv;
            last_signal = pid.update_with_measurement(error, pv, t0 + dt * i);
            let p = 0.01 * error;
            if i == 600 {
                integral_after_warmup = Some(last_signal - p);
            }
        }
        let final_error = 178.5 - (78.5 + 6000.0 * 0.1 * (3.0 / 60.0));
        let final_integral = last_signal - 0.01 * final_error;
        let start_integral = integral_after_warmup.unwrap();
        assert!(
            (final_integral - start_integral).abs() < 0.01,
            "integral should stay frozen while moving: {start_integral} -> {final_integral}"
        );
    }

    /// With stall integration disabled (the default), behaviour outside the band is the plain
    /// freeze — this is what keeps aquapath1 unchanged.
    #[test]
    fn stall_integration_is_off_by_default() {
        let mut pid = PidController::new(0.01, 0.00005, 0.0, -0.95, 0.95);
        let t0 = Instant::now();
        let dt = Duration::from_millis(100);
        let mut signal = 0.0;
        for i in 0..3000 {
            signal = pid.update_with_measurement(38.0, 140.0, t0 + dt * i);
        }
        let p_only = 0.01 * 38.0;
        assert!(
            (signal - p_only).abs() < 1e-6,
            "default must keep the plain freeze: signal={signal}, P alone={p_only}"
        );
    }

    #[test]
    fn reset_clears_derivative_state() {
        let mut pid = PidController::new(1.0, 0.1, 1.0, -1.0, 1.0);
        let t0 = Instant::now();
        pid.update_with_measurement(1.0, 10.0, t0);
        pid.update_with_measurement(1.0, 20.0, t0 + Duration::from_millis(100));
        pid.reset();
        // After a reset the next call is a "first update": proportional only.
        let signal = pid.update_with_measurement(2.0, 99.0, t0 + Duration::from_millis(200));
        assert!((signal - 2.0).abs() < 1e-9, "got {signal}");
    }
}
