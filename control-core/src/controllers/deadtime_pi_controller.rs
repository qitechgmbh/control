use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct TimeAgnosticDeadTimePiController {
    // Params
    /// Proportional gain
    kp: f64,
    /// Integral gain
    ki: f64,
    // State
    /// Proportional error
    ep: f64,
    /// Integral error
    ei: f64,
    /// Dead Time
    dead: Duration,

    last: Option<Instant>,
}

impl TimeAgnosticDeadTimePiController {
    pub fn new(kp: f64, ki: f64, dead: Duration) -> Self {
        Self {
            kp,
            ki,
            dead,
            ep: 0.0,
            ei: 0.0,
            last: None,
        }
    }

    pub fn configure(&mut self, kp: f64, ki: f64) {
        self.reset();
        self.kp = kp;
        self.ki = ki;
    }

    pub fn get_kp(&self) -> f64 {
        self.kp
    }

    pub fn get_ki(&self) -> f64 {
        self.ki
    }

    pub fn get_dead(&self) -> Duration {
        self.dead
    }

    pub fn set_kp(&mut self, kp: f64) {
        self.kp = kp;
    }

    pub fn set_ki(&mut self, ki: f64) {
        self.ki = ki;
    }

    pub fn set_dead(&mut self, dead: Duration) {
        self.dead = dead;
    }

    pub fn optional_clamp(value: f64, min: Option<f64>, max: Option<f64>) -> f64 {
        match (min, max) {
            (Some(min), Some(max)) => value.clamp(min, max),
            (Some(min), None) => value.max(min),
            (None, Some(max)) => value.min(max),
            (None, None) => value,
        }
    }

    pub fn update(&mut self, error: f64, t: Instant) -> f64 {
        let kp = self.kp / 10000.0;
        let ki = self.ki / 10000.0;
        let signal = match self.last {
            // First update
            None => {
                let ep = error;

                // Initialize integral error
                self.ei = 0.0;

                // Proportional + Integral
                let signal = kp * ep + ki * self.ei;

                // Save state
                self.ep = ep;
                self.last = Some(t);

                -signal
            }
            // Subsequent updates
            Some(last) => {
                // Calculate elapsed time
                let dt = t.duration_since(last).as_secs_f64();

                // Dead-time scaling factor
                let dead_secs = self.dead.as_secs_f64();
                let scale = (dt / dead_secs).min(1.0);

                // Smoothed proportional error
                let ep = self.ep + (error - self.ep) * scale;

                // Integrate error over time
                self.ei += error * dt;

                // Proportional + Integral
                let signal = kp * ep + ki * self.ei;

                // Save state
                self.ep = ep;
                self.last = Some(t);

                -signal
            }
        };
        -signal
    }

    pub fn reset(&mut self) {
        self.ep = 0.0;
        //self.last = None;
    }
}
