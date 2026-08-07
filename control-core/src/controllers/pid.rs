use std::time::Instant;

/// Band around the setpoint, in error units, inside which the integral is allowed to accumulate.
/// Outside it the accumulator is frozen (not reset), so a large approach does not wind it up.
const CONDITIONAL_INTEGRATION_BAND: f64 = 5.0;

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

                // Calculate signal
                let signal = self.kp * ep;

                // Set values
                self.ep = ep;
                self.ei = 0.0;

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

                let ei = if error.abs() < CONDITIONAL_INTEGRATION_BAND {
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
