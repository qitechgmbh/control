use std::time::Instant;

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
    /// Derivative error
    ed: f64,

    last: Option<Instant>,
}

impl PidController {
    pub const fn new(kp: f64, ki: f64, kd: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            ep: 0.0,
            ei: 0.0,
            ed: 0.0,
            last: None,
        }
    }

    pub const fn configure(&mut self, ki: f64, kp: f64, kd: f64) {
        self.reset();
        self.kp = kp;
        self.ki = ki;
        self.kd = kd;
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

    pub fn update(&mut self, error: f64, t: Instant) -> f64 {
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
                self.last = Some(t);

                signal
            }
            // Subsequent updates
            Some(last) => {
                // Calculate the time delta in seconds
                let dt = t.duration_since(last).as_secs_f64();

                // Calculate errors
                let ep = error;
                let ei = ep.mul_add(dt, self.ei);
                let ed = (ep - self.ep) / dt;

                // Calculate signal
                let signal = self.kd.mul_add(ed, self.kp.mul_add(ep, self.ki * ei));

                // Set values
                self.ep = ep;
                self.ei = ei;
                self.ed = ed;
                self.last = Some(t);

                signal
            }
        }
    }

    /// Like [`Self::update`], but with conditional-integration anti-windup.
    ///
    /// While the output is saturated and the error still points the same way,
    /// the integral is rolled back to its pre-tick value, so `ei` freezes
    /// instead of winding up. That makes a non-zero `ki` safe for plants that
    /// start far from the setpoint and saturate for a long time (e.g. the
    /// extruder heaters).
    ///
    /// `out_min`/`out_max` are the caller's own clamp bounds; the return value
    /// is [`Self::update`]'s signal clamped to them.
    pub fn update_with_antiwindup(
        &mut self,
        error: f64,
        t: Instant,
        out_min: f64,
        out_max: f64,
    ) -> f64 {
        debug_assert!(out_min <= out_max, "clamp bounds are inverted");

        // Snapshot rather than recomputing the increment: this stays correct
        // whatever integration rule `update` uses.
        let ei_before = self.ei;

        let clamped = self.update(error, t).clamp(out_min, out_max);

        let winding_up = clamped >= out_max && error > 0.0;
        let winding_down = clamped <= out_min && error < 0.0;
        if winding_up || winding_down {
            self.ei = ei_before;
        }

        clamped
    }

    pub const fn reset(&mut self) {
        self.ep = 0.0;
        self.ei = 0.0;
        self.ed = 0.0;
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Pure integrator, so the output is exactly `ki * ei` and the integral is
    /// easy to reason about.
    fn integrator() -> (PidController, Instant) {
        (PidController::new(0.0, 1.0, 0.0), Instant::now())
    }

    #[test]
    fn integral_freezes_while_saturated() {
        let (mut pid, t0) = integrator();
        // First tick establishes `last`; `update` zeroes `ei` on it either way.
        pid.update_with_antiwindup(10.0, t0, 0.0, 1.0);

        for i in 1..=10 {
            let out = pid.update_with_antiwindup(10.0, t0 + Duration::from_secs(i), 0.0, 1.0);
            assert_eq!(out, 1.0, "output should stay pinned at the upper bound");
        }
        assert_eq!(
            pid.ei, 0.0,
            "a saturated loop with same-sign error must not accumulate"
        );
    }

    /// The point of the whole thing: after a long saturated ramp, the loop must
    /// come off the rail as soon as the error reverses, rather than holding full
    /// output while it unwinds what it accumulated on the way up.
    #[test]
    fn leaves_the_rail_immediately_after_a_saturated_ramp() {
        let (mut anti, t0) = integrator();
        let mut wound = PidController::new(0.0, 1.0, 0.0);

        for i in 0..=10 {
            let t = t0 + Duration::from_secs(i);
            anti.update_with_antiwindup(10.0, t, 0.0, 1.0);
            wound.update(10.0, t);
        }

        // Setpoint reached and just passed.
        let t = t0 + Duration::from_secs(11);
        assert_eq!(
            anti.update_with_antiwindup(-1.0, t, 0.0, 1.0),
            0.0,
            "anti-windup: heater off the moment the zone is too hot"
        );
        assert!(
            wound.update(-1.0, t).clamp(0.0, 1.0) == 1.0,
            "without anti-windup the same loop is still at full power"
        );
    }

    /// A frozen integral is not a dead one — once the output is off the rail the
    /// term accumulates again.
    #[test]
    fn integral_resumes_once_the_output_is_unsaturated() {
        let (mut pid, t0) = integrator();
        for i in 0..=10 {
            pid.update_with_antiwindup(10.0, t0 + Duration::from_secs(i), 0.0, 1.0);
        }
        assert_eq!(pid.ei, 0.0, "frozen while saturated");

        // 0.5 K of error for one second leaves the output inside the range, so
        // there is nothing to roll back.
        let out = pid.update_with_antiwindup(0.5, t0 + Duration::from_secs(11), 0.0, 1.0);
        assert_eq!(pid.ei, 0.5);
        assert_eq!(out, 0.5);
    }

    #[test]
    fn unsaturated_output_integrates_normally() {
        let (mut pid, t0) = integrator();
        pid.update_with_antiwindup(0.1, t0, 0.0, 1.0);
        for i in 1..=5 {
            pid.update_with_antiwindup(0.1, t0 + Duration::from_secs(i), 0.0, 1.0);
        }
        assert!(
            (pid.ei - 0.5).abs() < 1e-12,
            "five seconds of 0.1 error, never clamped: {}",
            pid.ei
        );
    }

    #[test]
    fn saturating_at_the_lower_bound_freezes_too() {
        let (mut pid, t0) = integrator();
        pid.update_with_antiwindup(-10.0, t0, 0.0, 1.0);
        for i in 1..=5 {
            let out = pid.update_with_antiwindup(-10.0, t0 + Duration::from_secs(i), 0.0, 1.0);
            assert_eq!(out, 0.0);
        }
        assert_eq!(pid.ei, 0.0);
    }

    /// With `ki == 0` the integral is dead weight, so the rollback must not
    /// change the signal a caller sees.
    #[test]
    fn ki_zero_behaves_like_a_clamped_update() {
        let mut anti = PidController::new(0.5, 0.0, 0.0);
        let mut plain = PidController::new(0.5, 0.0, 0.0);
        let t0 = Instant::now();

        for i in 0..10u32 {
            let t = t0 + Duration::from_secs(u64::from(i));
            let error = 10.0 - f64::from(i);
            assert_eq!(
                anti.update_with_antiwindup(error, t, 0.0, 1.0),
                plain.update(error, t).clamp(0.0, 1.0)
            );
        }
    }

    #[test]
    fn reset_clears_the_frozen_state() {
        let (mut pid, t0) = integrator();
        pid.update_with_antiwindup(0.1, t0, 0.0, 1.0);
        pid.update_with_antiwindup(0.1, t0 + Duration::from_secs(1), 0.0, 1.0);
        assert_ne!(pid.ei, 0.0);

        pid.reset();
        assert_eq!(pid.ei, 0.0);
        assert!(pid.last.is_none());
    }
}
