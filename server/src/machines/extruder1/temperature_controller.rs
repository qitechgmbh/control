use std::time::{Duration, Instant};

use control_core::controllers::pid::PidController;

const PWM_PERIOD: Duration = Duration::from_millis(200);

#[derive(Debug)]

pub struct TemperatureController {
    pid: PidController,
    pub target_temp: f64,
    window_start: Instant,
}

impl TemperatureController {
    pub fn new(kp: f64, ki: f64, kd: f64, target_temp: f64) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd),
            target_temp,
            window_start: Instant::now(),
        }
    }

    pub fn update(&mut self, measured_temp: f64, now: Instant) -> bool {
        let error = self.target_temp - measured_temp;
        let control = self.pid.update(error, now); // PID output
        // println!("control {} error {}", control, error);
        // Clamp PID output to 0.0 – 1.0 (as duty cycle)
        let duty = control.clamp(0.0, 1.0);

        // Time since window started
        let elapsed = now.duration_since(self.window_start);

        // Restart window if needed
        if elapsed >= PWM_PERIOD {
            self.window_start = now;
        }

        // Compare duty cycle to elapsed time
        let on_time = PWM_PERIOD.mul_f64(duty);

        //println!("update Relais: {} duty: {}", elapsed < on_time, duty);
        // Relay is ON if within duty cycle window
        return elapsed < on_time;
    }
}
