use std::time::Instant;

#[derive(Debug)]
pub struct ClampingTimeagnosticPidController {
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

    min_ep: Option<f64>,
    max_ep: Option<f64>,

    min_ei: Option<f64>,
    max_ei: Option<f64>,

    min_ed: Option<f64>,
    max_ed: Option<f64>,

    min_signal: Option<f64>,
    max_signal: Option<f64>,

    last: Option<Instant>,
}

impl ClampingTimeagnosticPidController {
    pub const fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        min_ep: Option<f64>,
        max_ep: Option<f64>,
        min_ei: Option<f64>,
        max_ei: Option<f64>,
        min_ed: Option<f64>,
        max_ed: Option<f64>,
        min_signal: Option<f64>,
        max_signal: Option<f64>,
    ) -> Self {
        Self {
            kp,
            ki,
            kd,
            ep: 0.0,
            ei: 0.0,
            ed: 0.0,
            last: None,
            min_ep,
            max_ep,
            min_ei,
            max_ei,
            min_ed,
            max_ed,
            min_signal,
            max_signal,
        }
    }

    pub const fn simple_new(kp: f64, ki: f64, kd: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            ep: 0.0,
            ei: 0.0,
            ed: 0.0,
            min_ep: None,
            max_ep: None,
            min_ei: None,
            max_ei: None,
            min_ed: None,
            max_ed: None,
            last: None,
            min_signal: None,
            max_signal: None,
        }
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

    pub const fn configure(&mut self, ki: f64, kp: f64, kd: f64) {
        self.reset();
        self.kp = kp;
        self.ki = ki;
        self.kd = kd;
    }

    pub const fn optional_clamp(value: f64, min: Option<f64>, max: Option<f64>) -> f64 {
        match (min, max) {
            (Some(min), Some(max)) => value.clamp(min, max),
            (Some(min), None) => value.max(min),
            (None, Some(max)) => value.min(max),
            (None, None) => value,
        }
    }

    pub fn update(&mut self, error: f64, t: Instant) -> f64 {
        match self.last {
            // First update
            None => {
                // Calculate error
                let ep = error;

                // Calculate signal
                let signal = self.kp * ep;
                let clamped_signal = Self::optional_clamp(signal, self.min_signal, self.max_signal);

                // Set values
                self.ep = Self::optional_clamp(ep, self.min_ep, self.max_ep);
                self.ei = 0.0;
                self.ed = 0.0;
                self.last = Some(t);

                clamped_signal
            }
            // Subsequent updates
            Some(last) => {
                // Calculate the time delta in seconds
                let dt = t.duration_since(last).as_secs_f64();

                // Calculate errors
                let ep = Self::optional_clamp(error, self.min_ep, self.max_ep);

                let ei = Self::optional_clamp(ep.mul_add(dt, self.ei), self.min_ei, self.max_ei);

                let ed = Self::optional_clamp((ep - self.ep) / dt, self.min_ed, self.max_ed);

                // Make factors timeagnostic
                let kp = self.kp * dt;
                let ki = self.ki * dt;
                let kd = self.kd * dt;

                // Calculate signal
                let signal = kd.mul_add(ed, kp.mul_add(ep, ki * ei));
                let clamped_signal = Self::optional_clamp(signal, self.min_signal, self.max_signal);

                // Set values
                self.ep = ep;
                self.ei = ei;
                self.ed = ed;
                self.last = Some(t);

                clamped_signal
            }
        }
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
    use crate::controllers::clamping_timeagnostic_pid::ClampingTimeagnosticPidController;

    #[test]
    fn test_optional_clamp_with_both_bounds() {
        let val = 5.0;
        let clamped = ClampingTimeagnosticPidController::optional_clamp(val, Some(1.0), Some(4.0));
        assert_eq!(clamped, 4.0);

        let clamped = ClampingTimeagnosticPidController::optional_clamp(val, Some(6.0), Some(10.0));
        assert_eq!(clamped, 6.0);

        let clamped = ClampingTimeagnosticPidController::optional_clamp(val, Some(2.0), Some(6.0));
        assert_eq!(clamped, 5.0);
    }

    #[test]
    fn test_optional_clamp_with_only_min() {
        let val = 3.0;
        let clamped = ClampingTimeagnosticPidController::optional_clamp(val, Some(5.0), None);
        assert_eq!(clamped, 5.0);

        let clamped = ClampingTimeagnosticPidController::optional_clamp(val, Some(2.0), None);
        assert_eq!(clamped, 3.0);
    }

    #[test]
    fn test_optional_clamp_with_only_max() {
        let val = 7.0;
        let clamped = ClampingTimeagnosticPidController::optional_clamp(val, None, Some(6.0));
        assert_eq!(clamped, 6.0);

        let clamped = ClampingTimeagnosticPidController::optional_clamp(val, None, Some(8.0));
        assert_eq!(clamped, 7.0);
    }

    #[test]
    fn test_optional_clamp_with_no_bounds() {
        let val = 42.0;
        let clamped = ClampingTimeagnosticPidController::optional_clamp(val, None, None);
        assert_eq!(clamped, 42.0);
    }
}
