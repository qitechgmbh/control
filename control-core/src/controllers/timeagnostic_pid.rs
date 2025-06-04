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

    last: Option<Instant>,
}

impl TimeagnosticPidController {
    pub fn new(kp: f64, ki: f64, kd: f64) -> Self {
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
                let ep = error;
                let ei = self.ei + ep * dt;
                let ed = (ep - self.ep) / dt;

                // Make factors timeagnostic
                let kp = self.kp / dt;
                let ki = self.ki / dt;
                let kd = self.kd / dt;

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
