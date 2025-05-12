use std::time::{Duration, Instant};

use control_core::controllers::pid::PidController;

/// Clampable frequency limits (in Hz)
const MIN_FREQ: f64 = 0.0;
const MAX_FREQ: f64 = 60.0;

#[derive(Debug)]
pub struct PressureController {
    pid: PidController,
    pub target_pressure: f64,
    last_update: Instant,
}

impl PressureController {
    pub fn new(kp: f64, ki: f64, kd: f64, target_pressure: f64) -> Self {
        let now = Instant::now();
        Self {
            pid: PidController::new(kp, ki, kd),
            target_pressure,
            last_update: now,
        }
    }

    /// Returns the new frequency in Hz (clamped), based on pressure error
    pub fn update(&mut self, measured_pressure: f64, now: Instant) -> f64 {
        let error = self.target_pressure - measured_pressure;
        let freq = self.pid.update(error, now).clamp(MIN_FREQ, MAX_FREQ);
        self.last_update = now;
        freq
    }

    pub fn reset(&mut self) {
        self.pid.reset();
        self.last_update = Instant::now();
    }
}
