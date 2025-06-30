use std::time::Instant;

#[derive(Debug)]
pub struct TimeagnosticPidController {
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

    last: Option<Instant>,
}

impl TimeagnosticPidController {
    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        min_ep: Option<f64>,
        max_ep: Option<f64>,
        min_ei: Option<f64>,
        max_ei: Option<f64>,
        min_ed: Option<f64>,
        max_ed: Option<f64>,
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
        }
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
    }

    // Should this be moved to helpers?
    pub fn clamp_error(value: f64, min: Option<f64>, max: Option<f64>) -> f64 {
        match (min, max) {
            (Some(min), Some(max)) => value.clamp(min, max),
            (_, _) => value,
        }
    }

    pub fn simple_new(kp: f64, ki: f64, kd: f64) -> Self {
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
        }
    }

    pub fn update(&mut self, error: f64, t: Instant) -> f64 {
        let signal = match self.last {
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
                let ep = TimeagnosticPidController::clamp_error(error, self.min_ep, self.max_ep);
                let ei = TimeagnosticPidController::clamp_error(
                    self.ei + ep * dt,
                    self.min_ei,
                    self.max_ei,
                );

                let ed = TimeagnosticPidController::clamp_error(
                    (ep - self.ep) / dt,
                    self.min_ed,
                    self.max_ed,
                );

                // Make factors timeagnostic
                let kp = self.kp * dt;
                let ki = self.ki * dt;
                let kd = self.kd * dt;

                // Calculate signal
                let signal = kp * ep + ki * ei + kd * ed;

                // Set values
                self.ep = ep;
                self.ei = ei;
                self.ed = ed;
                self.last = Some(t);

                signal
            }
        };

        signal
    }

    pub fn reset(&mut self) {
        self.ep = 0.0;
        self.ei = 0.0;
        self.ed = 0.0;
        self.last = None;
    }
}
