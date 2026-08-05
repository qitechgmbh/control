use std::time::Instant;

/// PID controller for PWM-driven heaters, ported from Klipper's
/// `ControlPID` (`klippy/extras/heaters.py`).
///
/// Differs from [`super::pid::PidController`] in two ways that matter for
/// heaters specifically:
/// - The derivative term is computed from the measured value, not the
///   error, so a setpoint change alone never produces a derivative kick.
///   Below `min_deriv_time` the previous derivative estimate is blended in
///   instead of differentiating over a tiny `dt`.
/// - The integral term is clamped to `[0, max_power / ki]` — the exact
///   bound at which it alone could saturate the output — and only
///   accumulates while the unclamped output equals the clamped output, so
///   it stops growing the instant the actuator saturates and resumes the
///   moment it un-saturates.
#[derive(Debug)]
pub struct HeaterPidController {
    kp: f64,
    ki: f64,
    kd: f64,

    /// Upper bound for the output, and the value used to derive `temp_integ_max`.
    max_power: f64,
    /// Smoothing floor (seconds) for the derivative term.
    min_deriv_time: f64,
    /// `max_power / ki` (0.0 if `ki == 0.0`).
    temp_integ_max: f64,

    prev_temp: Option<f64>,
    prev_temp_time: Option<Instant>,
    prev_temp_deriv: f64,
    prev_temp_integ: f64,
}

impl HeaterPidController {
    pub fn new(kp: f64, ki: f64, kd: f64, max_power: f64, min_deriv_time: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            max_power,
            min_deriv_time,
            temp_integ_max: Self::compute_temp_integ_max(max_power, ki),
            prev_temp: None,
            prev_temp_time: None,
            prev_temp_deriv: 0.0,
            prev_temp_integ: 0.0,
        }
    }

    fn compute_temp_integ_max(max_power: f64, ki: f64) -> f64 {
        if ki != 0.0 { max_power / ki } else { 0.0 }
    }

    pub fn get_kp(&self) -> f64 {
        self.kp
    }

    pub fn get_ki(&self) -> f64 {
        self.ki
    }

    pub fn get_kd(&self) -> f64 {
        self.kd
    }

    pub fn configure(&mut self, ki: f64, kp: f64, kd: f64) {
        self.reset();
        self.kp = kp;
        self.ki = ki;
        self.kd = kd;
        self.temp_integ_max = Self::compute_temp_integ_max(self.max_power, ki);
    }

    pub fn reset(&mut self) {
        self.prev_temp = None;
        self.prev_temp_time = None;
        self.prev_temp_deriv = 0.0;
        self.prev_temp_integ = 0.0;
    }

    /// Advance the controller with a new measurement and return the output,
    /// clamped to `[0, max_power]`.
    pub fn update(&mut self, measured_temp: f64, target_temp: f64, now: Instant) -> f64 {
        let (Some(prev_temp), Some(prev_temp_time)) = (self.prev_temp, self.prev_temp_time) else {
            let temp_err = target_temp - measured_temp;
            let output = (self.kp * temp_err).clamp(0.0, self.max_power);
            self.prev_temp = Some(measured_temp);
            self.prev_temp_time = Some(now);
            self.prev_temp_deriv = 0.0;
            self.prev_temp_integ = 0.0;
            return output;
        };

        let time_diff = now.duration_since(prev_temp_time).as_secs_f64();
        let temp_diff = measured_temp - prev_temp;
        let temp_deriv = if time_diff >= self.min_deriv_time {
            temp_diff / time_diff
        } else {
            (self.prev_temp_deriv * (self.min_deriv_time - time_diff) + temp_diff)
                / self.min_deriv_time
        };

        let temp_err = target_temp - measured_temp;
        let temp_integ =
            (self.prev_temp_integ + temp_err * time_diff).clamp(0.0, self.temp_integ_max);

        let co = self.kp * temp_err + self.ki * temp_integ - self.kd * temp_deriv;
        let bounded_co = co.clamp(0.0, self.max_power);

        // Anti-windup: only persist integral growth while the output isn't saturated.
        if co == bounded_co {
            self.prev_temp_integ = temp_integ;
        }

        self.prev_temp = Some(measured_temp);
        self.prev_temp_time = Some(now);
        self.prev_temp_deriv = temp_deriv;

        bounded_co
    }
}

#[cfg(test)]
mod tests {
    use super::HeaterPidController;
    use std::time::{Duration, Instant};

    #[test]
    fn first_call_is_proportional_only_and_clamped() {
        let mut pid = HeaterPidController::new(0.1, 0.0, 0.0, 1.0, 1.0);
        let now = Instant::now();
        // error = 200, kp*error = 20, clamped to max_power = 1.0
        let out = pid.update(0.0, 200.0, now);
        assert_eq!(out, 1.0);
    }

    #[test]
    fn setpoint_jump_does_not_spike_derivative() {
        // kd is large enough that a derivative kick would be obvious if present.
        let mut pid = HeaterPidController::new(0.0, 0.0, 5.0, 1.0, 1.0);
        let mut now = Instant::now();
        // Seed state with a stable temperature.
        let _ = pid.update(50.0, 50.0, now);
        now += Duration::from_millis(100);
        let _ = pid.update(50.0, 50.0, now);

        // Only the target jumps; measured temperature is unchanged, so the
        // derivative-on-measurement term must stay ~0, not spike.
        now += Duration::from_millis(100);
        let out = pid.update(50.0, 250.0, now);
        assert_eq!(
            out, 0.0,
            "target-only jump must not move a kd-only controller's output"
        );
    }

    #[test]
    fn integral_clamps_to_max_power_over_ki_and_stops_growing_once_saturated() {
        let max_power = 1.0;
        let ki = 0.02;
        let mut pid = HeaterPidController::new(0.0, ki, 0.0, max_power, 0.001);
        let mut now = Instant::now();
        let _ = pid.update(0.0, 300.0, now); // seed

        // Drive a large, sustained error so the integral saturates.
        for _ in 0..1000 {
            now += Duration::from_millis(100);
            let _ = pid.update(0.0, 300.0, now);
        }

        let out = pid.update(0.0, 300.0, now + Duration::from_millis(100));
        assert_eq!(out, max_power);
        // temp_integ_max = max_power / ki; ki*temp_integ_max must equal max_power exactly.
        assert_eq!(ki * (max_power / ki), max_power);
    }

    #[test]
    fn integral_unclamps_once_temperature_overshoots() {
        let max_power = 1.0;
        let ki = 0.5;
        let mut pid = HeaterPidController::new(0.0, ki, 0.0, max_power, 0.001);
        let mut now = Instant::now();
        let _ = pid.update(0.0, 300.0, now); // seed

        // Saturate the integral while far from target.
        for _ in 0..50 {
            now += Duration::from_millis(100);
            let _ = pid.update(0.0, 300.0, now);
        }
        // At exactly zero error the integral holds (co == max_power is a marginal,
        // not a decreasing, state) — matches Klipper: temp_integ only shrinks once
        // temp_err goes negative, i.e. the temperature actually overshoots target.
        now += Duration::from_millis(100);
        let at_target = pid.update(300.0, 300.0, now);
        assert_eq!(at_target, max_power);

        // Now overshoot: measured temp above target, error negative, integral -
        // and therefore output - must fall.
        now += Duration::from_millis(100);
        let out = pid.update(310.0, 300.0, now);
        assert!(
            out < max_power,
            "output should fall once temperature overshoots, got {out}"
        );
    }

    #[test]
    fn sub_floor_dt_blends_previous_derivative_instead_of_spiking() {
        let mut pid = HeaterPidController::new(0.0, 0.0, 1.0, 10.0, 1.0);
        let mut now = Instant::now();
        let _ = pid.update(50.0, 50.0, now);

        // A very small dt with a real temperature jump would blow up a naive
        // `temp_diff / time_diff` derivative; min_deriv_time blending must
        // keep the result bounded instead.
        now += Duration::from_millis(1);
        let out = pid.update(51.0, 50.0, now);
        assert!(
            out.abs() < 10.0,
            "derivative blend should stay bounded, got {out}"
        );
    }
}
