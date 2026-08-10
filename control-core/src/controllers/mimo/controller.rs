//! Matrix-gain PID controller.
//!
//! Deliberately mirrors [`crate::controllers::pid::PidController`] term for term, so that a
//! [`MimoPidController`] configured with diagonal gain matrices behaves identically to a bank of
//! independent SISO loops. That equivalence is what makes it safe to switch a running machine
//! between the two: the MIMO path is a strict generalisation, not a different controller.
//!
//! What genuinely differs is saturation handling. Scalar back-calculation does not extend to
//! matrix gains, because the integral state lives in error-space while the limits apply in
//! actuator-space. See [`MimoPidController::update`].

use super::matrix::{self, Mat, Vec_};
use std::time::Instant;

/// Error band, in process units, outside which integration is frozen.
///
/// Same value and same reasoning as the SISO controller: during a large setpoint change the
/// integral would otherwise accumulate the entire transient, and the loop would overshoot by
/// however long it took to get there.
pub const DEFAULT_INTEGRATION_BAND: f64 = 15.0;

/// Matrix-gain PID with per-output saturation and directional anti-windup.
#[derive(Debug, Clone)]
pub struct MimoPidController<const N: usize> {
    kp: Mat<N>,
    ki: Mat<N>,
    kd: Mat<N>,

    /// Integrated error per channel.
    integral: Vec_<N>,
    /// Filtered derivative of the measurement, per channel.
    derivative: Vec_<N>,
    last_measurement: Option<Vec_<N>>,
    last_update: Option<Instant>,

    derivative_filter_tc: f64,
    integration_band: f64,
    u_min: Vec_<N>,
    u_max: Vec_<N>,

    /// Last commanded output, retained for reporting.
    last_output: Vec_<N>,
}

impl<const N: usize> MimoPidController<N> {
    pub fn new(kp: Mat<N>, ki: Mat<N>, kd: Mat<N>, u_min: Vec_<N>, u_max: Vec_<N>) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: [0.0; N],
            derivative: [0.0; N],
            last_measurement: None,
            last_update: None,
            derivative_filter_tc: 0.0,
            integration_band: DEFAULT_INTEGRATION_BAND,
            u_min,
            u_max,
            last_output: [0.0; N],
        }
    }

    /// Build from synthesized gains, with outputs clamped to `[0, max_duty_i]`.
    pub fn from_gains(kp: Mat<N>, ki: Mat<N>, kd: Mat<N>, filter_tc: f64, u_max: Vec_<N>) -> Self {
        let mut c = Self::new(kp, ki, kd, [0.0; N], u_max);
        c.derivative_filter_tc = filter_tc;
        c
    }

    pub fn configure(&mut self, kp: Mat<N>, ki: Mat<N>, kd: Mat<N>, filter_tc: f64) {
        self.kp = kp;
        self.ki = ki;
        self.kd = kd;
        self.derivative_filter_tc = filter_tc.max(0.0);
    }

    /// Set the derivative filter time constant in seconds. `0.0` disables filtering.
    ///
    /// Does not reset the controller — the filter can be retuned without disturbing the integral.
    pub fn set_derivative_filter(&mut self, tc: f64) {
        self.derivative_filter_tc = if tc > 0.0 { tc } else { 0.0 };
    }

    pub fn set_integration_band(&mut self, band: f64) {
        self.integration_band = band.max(0.0);
    }

    pub fn set_output_limits(&mut self, u_min: Vec_<N>, u_max: Vec_<N>) {
        self.u_min = u_min;
        self.u_max = u_max;
    }

    /// Seed the integral so that it alone produces `u` at zero error.
    ///
    /// Used when switching a warm machine onto MIMO control: without this the output would jump to
    /// the proportional term only, the heaters would drop out, and the barrel would sag before the
    /// integral rebuilt the standing load. Solves `KI * integral = u` in the least-squares sense.
    pub fn preload_output(&mut self, u: &Vec_<N>) {
        if let Some(pinv) = matrix::pseudo_inverse(&self.ki) {
            let seed = matrix::matvec(&pinv, u);
            if seed.iter().all(|v| v.is_finite()) {
                self.integral = seed;
            }
        }
    }

    pub fn reset(&mut self) {
        self.integral = [0.0; N];
        self.derivative = [0.0; N];
        self.last_measurement = None;
        self.last_update = None;
        self.last_output = [0.0; N];
    }

    pub const fn last_output(&self) -> &Vec_<N> {
        &self.last_output
    }

    pub const fn integral(&self) -> &Vec_<N> {
        &self.integral
    }

    /// Compute this tick's outputs.
    ///
    /// Term structure matches the SISO controller: proportional and integral act on the error,
    /// while the derivative acts on the *measurement* so that moving a setpoint does not kick the
    /// output. Each term is formed per channel and then mixed by its gain matrix — that ordering
    /// is what makes diagonal matrices collapse exactly onto `N` independent loops.
    ///
    /// # Saturation
    ///
    /// Outputs are clamped per channel, then the clamped-away part is projected back into
    /// error-space through `pinv(KI)` and removed from the integral. This is the matrix
    /// generalisation of scalar back-calculation: with a diagonal `KI` it reduces exactly to the
    /// per-channel form, and with a coupled `KI` it unwinds the specific combination of integrator
    /// states that produced the excess, rather than every state that happens to feed a saturated
    /// output.
    pub fn update(&mut self, setpoints: &Vec_<N>, measurements: &Vec_<N>, now: Instant) -> Vec_<N> {
        let dt = match self.last_update {
            Some(prev) => now.saturating_duration_since(prev).as_secs_f64(),
            None => {
                // First tick: seed the derivative history and command the proportional term only.
                self.last_update = Some(now);
                self.last_measurement = Some(*measurements);
                let mut e = [0.0; N];
                for i in 0..N {
                    e[i] = setpoints[i] - measurements[i];
                }
                let u = self.clamp(&matrix::matvec(&self.kp, &e));
                self.last_output = u;
                return u;
            }
        };

        // The act loop is free-running and can tick twice within the clock's resolution. A zero or
        // negative dt would divide by zero in the derivative and poison every downstream term with
        // NaN, which no amount of clamping recovers from.
        if dt <= 0.0 {
            return self.last_output;
        }
        self.last_update = Some(now);

        let mut error = [0.0; N];
        for i in 0..N {
            error[i] = setpoints[i] - measurements[i];
        }

        // Derivative on measurement, negated so it opposes a rising process variable.
        let prev = self.last_measurement.unwrap_or(*measurements);
        let alpha = if self.derivative_filter_tc > 0.0 {
            dt / (self.derivative_filter_tc + dt)
        } else {
            1.0
        };
        for i in 0..N {
            let raw = -(measurements[i] - prev[i]) / dt;
            // `mul_add` and the strict `<` below both mirror `PidController` exactly, so the two
            // controllers agree to well under a rounding step while neither is saturated.
            self.derivative[i] = alpha.mul_add(raw - self.derivative[i], self.derivative[i]);
        }
        self.last_measurement = Some(*measurements);

        // Integrate only the channels that are inside the band.
        for i in 0..N {
            if error[i].abs() < self.integration_band {
                self.integral[i] = error[i].mul_add(dt, self.integral[i]);
            }
        }

        let unclamped = self.mix(&error);
        let clamped = self.clamp(&unclamped);

        let mut excess = [0.0; N];
        let mut saturated = false;
        for i in 0..N {
            excess[i] = unclamped[i] - clamped[i];
            if excess[i] != 0.0 {
                saturated = true;
            }
        }

        if saturated {
            match matrix::pseudo_inverse(&self.ki) {
                Some(pinv) => {
                    let correction = matrix::matvec(&pinv, &excess);
                    if correction.iter().all(|v| v.is_finite()) {
                        for i in 0..N {
                            self.integral[i] -= correction[i];
                        }
                    }
                }
                None => {
                    // No usable KI to project through - undo this tick's integration outright
                    // rather than let it accumulate against a limit it cannot move.
                    for i in 0..N {
                        if error[i].abs() <= self.integration_band {
                            self.integral[i] -= error[i] * dt;
                        }
                    }
                }
            }
        }

        self.last_output = clamped;
        clamped
    }

    fn mix(&self, error: &Vec_<N>) -> Vec_<N> {
        let p = matrix::matvec(&self.kp, error);
        let i = matrix::matvec(&self.ki, &self.integral);
        let d = matrix::matvec(&self.kd, &self.derivative);
        let mut out = [0.0; N];
        for k in 0..N {
            out[k] = p[k] + i[k] + d[k];
        }
        out
    }

    fn clamp(&self, u: &Vec_<N>) -> Vec_<N> {
        let mut out = *u;
        for i in 0..N {
            out[i] = if out[i].is_finite() {
                out[i].clamp(self.u_min[i], self.u_max[i])
            } else {
                self.u_min[i]
            };
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controllers::pid::PidController;
    use std::time::Duration;

    fn diag4(v: [f64; 4]) -> Mat<4> {
        matrix::diag(&v)
    }

    /// A MIMO controller with diagonal gains must be indistinguishable from four SISO controllers.
    /// This is the property that makes switching a live machine between the two schemes safe.
    ///
    /// Checked to a tolerance rather than to the bit, for one specific reason: `PidController`
    /// round-trips its accumulator through `ei = (ki * ei) / ki` every tick as part of its
    /// anti-windup, and that is not an exact identity in binary floating point. The two therefore
    /// drift apart in the last bits over a long run even when performing identical arithmetic.
    /// Every structural mistake this test exists to catch — a transposed matrix, terms mixed in
    /// the wrong order, an unfiltered derivative, a band comparison off by an equals sign — moves
    /// the output by orders of magnitude more than the tolerance below.
    ///
    /// The run is deliberately kept inside the linear region. Once either controller saturates the
    /// two *should* diverge: that is the deliberate upgrade, covered by the anti-windup tests.
    #[test]
    fn mimo_matches_four_sisos_when_diagonal_and_unsaturated() {
        let kp = [0.016, 0.020, 0.012, 0.030];
        let ki = [0.001, 0.0015, 0.0008, 0.002];
        let kd = [0.008, 0.010, 0.006, 0.012];
        let limit = |i: usize| if i == 3 { 0.95 } else { 1.0 };

        let mut mimo = MimoPidController::<4>::new(
            diag4(kp),
            diag4(ki),
            diag4(kd),
            [0.0; 4],
            [limit(0), limit(1), limit(2), limit(3)],
        );
        mimo.set_derivative_filter(1.0);

        let mut sisos: Vec<PidController> = (0..4)
            .map(|i| {
                let mut p = PidController::new(kp[i], ki[i], kd[i], -limit(i), limit(i));
                p.set_derivative_filter(1.0);
                p
            })
            .collect();

        // Start a few degrees below setpoint so the error is non-zero but inside the 15-degree
        // integration band. The integral then builds towards the standing load and the outputs
        // spend the whole run strictly between their limits, which is the regime where the two
        // controllers are supposed to agree.
        let base = [200.0, 195.0, 190.0, 210.0];
        let mut pv = [195.0, 190.0, 185.0, 205.0];
        let mut t = Instant::now();
        let mut worst = 0.0_f64;

        for step in 0..4000 {
            t += Duration::from_millis(250);
            // Small, slow setpoint motion, kept well inside the band.
            let wobble = 2.0 * (step as f64 / 250.0).sin();
            let setpoints = [
                base[0] + wobble,
                base[1] - wobble,
                base[2] + 0.5 * wobble,
                base[3],
            ];

            let m = mimo.update(&setpoints, &pv, t);
            for i in 0..4 {
                let s = sisos[i]
                    .update_with_measurement(setpoints[i] - pv[i], pv[i], t)
                    .clamp(0.0, limit(i));
                worst = worst.max((m[i] - s).abs());
                assert!(
                    m[i] > 0.0 && m[i] < limit(i),
                    "channel {i} saturated at step {step} ({}), which would make this \
                     comparison vacuous",
                    m[i]
                );
            }

            // First-order plant per channel, no cross-coupling.
            for i in 0..4 {
                pv[i] += (m[i] * 400.0 - (pv[i] - 20.0)) * 0.25 / 300.0;
            }
        }

        assert!(
            worst < 1e-9,
            "diagonal MIMO drifted from the SISO bank by {worst}"
        );
    }

    #[test]
    fn off_diagonal_gains_actually_couple_the_channels() {
        // A pure off-diagonal KP must make channel 0's error drive output 1.
        let mut kp = matrix::zeros::<2>();
        kp[1][0] = 0.5;
        let mut c =
            MimoPidController::<2>::new(kp, matrix::zeros(), matrix::zeros(), [0.0; 2], [1.0; 2]);

        let t0 = Instant::now();
        c.update(&[1.0, 0.0], &[0.0, 0.0], t0);
        let u = c.update(&[1.0, 0.0], &[0.0, 0.0], t0 + Duration::from_millis(100));

        assert_eq!(u[0], 0.0, "channel 0 has no gain of its own");
        assert!(
            (u[1] - 0.5).abs() < 1e-12,
            "expected 0.5 * 1.0, got {}",
            u[1]
        );
    }

    #[test]
    fn zero_dt_does_not_poison_the_output() {
        let mut c =
            MimoPidController::<2>::new(diag2(0.5), diag2(0.01), diag2(0.1), [0.0; 2], [1.0; 2]);
        let t = Instant::now();
        c.update(&[10.0, 10.0], &[0.0, 0.0], t);
        let a = c.update(&[10.0, 10.0], &[0.0, 0.0], t + Duration::from_millis(100));
        // Same instant twice: must return the previous output untouched, not NaN.
        let b = c.update(&[10.0, 10.0], &[0.0, 0.0], t + Duration::from_millis(100));
        assert_eq!(a, b);
        assert!(b.iter().all(|v| v.is_finite()));
    }

    fn diag2(v: f64) -> Mat<2> {
        matrix::diag(&[v, v])
    }

    #[test]
    fn antiwindup_holds_under_full_saturation() {
        // Every output pinned at its limit against an error it cannot clear. The integral must
        // stay bounded, otherwise recovery is delayed by however long the run lasted.
        let mut c = MimoPidController::<4>::new(
            diag4([0.5; 4]),
            diag4([0.05; 4]),
            matrix::zeros(),
            [0.0; 4],
            [1.0; 4],
        );
        c.set_integration_band(1e9); // force integration despite the huge error

        let mut t = Instant::now();
        let mut early = [0.0; 4];
        for k in 0..20_000 {
            t += Duration::from_millis(100);
            let u = c.update(&[300.0; 4], &[20.0; 4], t);
            assert!(u.iter().all(|&v| (0.0..=1.0).contains(&v)));
            if k == 1_000 {
                early = *c.integral();
            }
        }

        // The property that matters is not a magnitude but that the integral stops growing: it
        // settles at whatever value holds the output exactly at the limit, and stays there however
        // long the run continues. Without back-calculation it would grow without bound with the
        // length of the saturation, and recovery would be delayed by the same amount.
        let late = *c.integral();
        for i in 0..4 {
            assert!(
                (late[i] - early[i]).abs() < 1e-6,
                "channel {i} integral still moving between tick 1000 ({}) and 20000 ({})",
                early[i],
                late[i]
            );
            assert!(late[i].is_finite());
        }
    }

    #[test]
    fn antiwindup_recovers_promptly_once_the_setpoint_is_reachable() {
        let mut c = MimoPidController::<2>::new(
            diag2(0.02),
            diag2(0.002),
            matrix::zeros(),
            [0.0; 2],
            [1.0; 2],
        );
        c.set_integration_band(1e9);

        let mut t = Instant::now();
        // Drive hard against an unreachable setpoint.
        for _ in 0..5000 {
            t += Duration::from_millis(100);
            c.update(&[500.0, 500.0], &[20.0, 20.0], t);
        }
        // Now the process arrives at a reachable setpoint; the output must come off the rail
        // within a few ticks rather than staying saturated while a wound-up integral unwinds.
        let mut ticks_to_leave_rail = None;
        for k in 0..2000 {
            t += Duration::from_millis(100);
            let u = c.update(&[100.0, 100.0], &[100.0, 100.0], t);
            if u[0] < 0.999 {
                ticks_to_leave_rail = Some(k);
                break;
            }
        }
        assert!(
            ticks_to_leave_rail.is_some_and(|k| k < 200),
            "took {ticks_to_leave_rail:?} ticks to come off the rail"
        );
    }

    #[test]
    fn integration_band_freezes_the_integral_on_a_large_error() {
        // Gains chosen so the output stays well inside its limits: this isolates the band from
        // the anti-windup, which also writes to the integral.
        let mut c = MimoPidController::<2>::new(
            diag2(0.001),
            diag2(0.01),
            matrix::zeros(),
            [0.0; 2],
            [1.0; 2],
        );
        c.set_integration_band(15.0);

        let mut t = Instant::now();
        c.update(&[200.0, 200.0], &[20.0, 20.0], t);
        for _ in 0..100 {
            t += Duration::from_millis(100);
            let u = c.update(&[200.0, 200.0], &[20.0, 20.0], t);
            assert!(
                u.iter().all(|&v| v > 0.0 && v < 1.0),
                "test setup saturated"
            );
        }
        assert_eq!(
            c.integral(),
            &[0.0, 0.0],
            "a 180-degree error is far outside the band and must not integrate"
        );
    }

    #[test]
    fn integration_band_admits_a_small_error() {
        // The mirror of the test above: inside the band the integral must accumulate, otherwise
        // the band would simply be disabling integral action altogether.
        let mut c = MimoPidController::<2>::new(
            diag2(0.001),
            diag2(0.01),
            matrix::zeros(),
            [0.0; 2],
            [1.0; 2],
        );
        c.set_integration_band(15.0);

        let mut t = Instant::now();
        c.update(&[205.0, 205.0], &[200.0, 200.0], t);
        for _ in 0..100 {
            t += Duration::from_millis(100);
            c.update(&[205.0, 205.0], &[200.0, 200.0], t);
        }
        // 5 degrees of error over 100 ticks of 0.1 s.
        for i in 0..2 {
            assert!(
                (c.integral()[i] - 50.0).abs() < 1e-9,
                "channel {i} integral {} should be ~50",
                c.integral()[i]
            );
        }
    }

    #[test]
    fn preload_reproduces_the_requested_output_at_zero_error() {
        let mut c = MimoPidController::<4>::new(
            diag4([0.1; 4]),
            diag4([0.01, 0.02, 0.005, 0.03]),
            matrix::zeros(),
            [0.0; 4],
            [1.0; 4],
        );
        let hold = [0.35, 0.40, 0.28, 0.52];
        c.preload_output(&hold);

        let t = Instant::now();
        c.update(&[200.0; 4], &[200.0; 4], t);
        let u = c.update(&[200.0; 4], &[200.0; 4], t + Duration::from_millis(100));
        for i in 0..4 {
            assert!(
                (u[i] - hold[i]).abs() < 1e-6,
                "channel {i}: {} vs {}",
                u[i],
                hold[i]
            );
        }
    }

    #[test]
    fn preload_works_through_a_coupled_ki() {
        let ki: Mat<2> = [[0.02, 0.006], [0.005, 0.03]];
        let mut c =
            MimoPidController::<2>::new(diag2(0.0), ki, matrix::zeros(), [0.0; 2], [1.0; 2]);
        let hold = [0.42, 0.31];
        c.preload_output(&hold);

        let t = Instant::now();
        c.update(&[200.0; 2], &[200.0; 2], t);
        let u = c.update(&[200.0; 2], &[200.0; 2], t + Duration::from_millis(100));
        for i in 0..2 {
            assert!((u[i] - hold[i]).abs() < 1e-6, "channel {i}: {}", u[i]);
        }
    }

    #[test]
    fn outputs_never_leave_their_limits() {
        let mut c = MimoPidController::<4>::new(
            diag4([5.0; 4]),
            diag4([1.0; 4]),
            diag4([2.0; 4]),
            [0.0; 4],
            [1.0, 1.0, 1.0, 0.95],
        );
        let mut t = Instant::now();
        let mut pv = [20.0; 4];
        for k in 0..5000 {
            t += Duration::from_millis(100);
            // Swing the setpoint around to exercise both rails.
            let sp = if (k / 500) % 2 == 0 { 300.0 } else { 0.0 };
            let u = c.update(&[sp; 4], &pv, t);
            for i in 0..4 {
                let limit = if i == 3 { 0.95 } else { 1.0 };
                assert!(
                    (0.0..=limit).contains(&u[i]),
                    "channel {i} output {} left [0, {limit}] at step {k}",
                    u[i]
                );
                pv[i] += (u[i] * 400.0 - (pv[i] - 20.0)) * 0.1 / 200.0;
            }
        }
    }

    #[test]
    fn reset_clears_all_state() {
        let mut c =
            MimoPidController::<2>::new(diag2(0.1), diag2(0.01), diag2(0.05), [0.0; 2], [1.0; 2]);
        let mut t = Instant::now();
        for _ in 0..50 {
            t += Duration::from_millis(100);
            c.update(&[25.0, 25.0], &[20.0, 20.0], t);
        }
        assert!(c.integral().iter().any(|v| *v != 0.0));
        c.reset();
        assert_eq!(c.integral(), &[0.0, 0.0]);
        assert_eq!(c.last_output(), &[0.0, 0.0]);
    }
}
