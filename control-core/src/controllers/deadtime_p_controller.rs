use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct TimeAgnosticDeadTimePController {
    // Params
    /// Proportional gain
    kp: f64,
    // State
    /// Proportional error
    ep: f64,
    /// Dead Time
    dead: Duration,

    last: Option<Instant>,
}

impl TimeAgnosticDeadTimePController {
    pub fn new(kp: f64, dead: Duration) -> Self {
        Self {
            kp,
            dead,
            ep: 0.0,
            last: None,
        }
    }

    pub fn configure(&mut self, kp: f64) {
        self.reset();
        self.kp = kp;
    }

    pub fn get_kp(&self) -> f64 {
        self.kp
    }

    pub fn get_dead(&self) -> Duration {
        self.dead
    }

    pub fn set_kp(&mut self, kp: f64) {
        self.kp = kp;
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
        // Only update internal state every `update_interval` cycles
        let signal = match self.last {
            // First update
            None => {
                // Calculate error
                let ep = error;

                // Calculate signal
                let signal = self.kp * ep;

                // Set values
                self.ep = ep;

                self.last = Some(t);

                -signal
            }
            // Subsequent updates
            Some(last) => {
                // Calculate the time delta in seconds
                let dt = t.duration_since(last).as_secs_f64();

                // Dead-time scaling factor
                let dead_secs = self.dead.as_secs_f64();
                let scale = (dt / dead_secs).min(1.0);

                // Calculate errors
                let ep = self.ep + (error - self.ep) * scale;

                // Calculate signal
                let signal = self.kp * ep;

                // Set values
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
