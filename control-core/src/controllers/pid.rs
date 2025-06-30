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

    /// Prevents a common issue with PID's known as integral windup
    /// This issue can cause PID's to overshoot drastically for some time until the "windup"
    /// has "decayed" over time : Basically clamps ei to a min and max
    integral_windup_clamp_max: f64,
    integral_windup_clamp_min: f64,
    last: Option<Instant>,
}

impl PidController {
    pub fn new(kp: f64, ki: f64, kd: f64, max: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            ep: 0.0,
            ei: 0.0,
            ed: 0.0,
            last: None,
            integral_windup_clamp_max: max,
            integral_windup_clamp_min: 0.0,
        }
    }

    pub fn configure(&mut self, ki: f64, kp: f64, kd: f64) {
        self.reset();
        self.kp = kp;
        self.ki = ki;
        self.kd = kd;
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
                let ei = (self.ei + ep * dt).clamp(
                    self.integral_windup_clamp_min,
                    self.integral_windup_clamp_max,
                );

                let ed = (ep - self.ep) / dt;

                // Calculate signal
                let signal = self.kp * ep + self.ki * ei + self.kd * ed;

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
