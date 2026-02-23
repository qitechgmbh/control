//! PID Auto-tuner based on Klipper's algorithm
//!
//! This implementation uses the Astrom-Hagglund relay method to identify
//! system oscillations, then applies Ziegler-Nichols tuning rules to
//! calculate PID parameters.
//!
//! The algorithm works by:
//! 1. Driving the output to max when the measurement is below `target - delta`,
//!    then to zero when above `target + delta`
//! 2. Recording measurement peaks during these oscillations
//! 3. Calculating the ultimate gain (Ku) and period (Tu) from peak data
//! 4. Applying Ziegler-Nichols rules to derive Kp, Ki, Kd values
//!
//! This tuner is generic and can be used for any physical quantity (pressure in
//! bar, temperature in °C, etc.).  The caller is responsible for mapping the
//! returned duty-cycle (0.0 … max_power) to the actual actuator output.

use std::time::Instant;

/// Configuration for PID auto-tuning
#[derive(Debug, Clone)]
pub struct AutoTuneConfig {
    /// Oscillation amplitude around the setpoint (e.g. ±5 °C or ±0.5 bar)
    pub tune_delta: f64,
    /// Maximum output fraction (0.0 … 1.0).  Maps to "full on" in bang-bang
    /// control during the tuning phase.
    pub max_power: f64,
    /// Maximum duration for auto-tuning in seconds before the run is aborted
    pub max_duration_secs: f64,
}

impl Default for AutoTuneConfig {
    fn default() -> Self {
        Self {
            tune_delta: 5.0,
            max_power: 1.0,
            max_duration_secs: 3600.0, // 60 minutes
        }
    }
}

/// Result of PID auto-tuning
#[derive(Debug, Clone, PartialEq)]
pub struct AutoTuneResult {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    /// Ultimate gain (Ku) used in the Ziegler–Nichols calculation
    pub ku: f64,
    /// Ultimate period (Tu) used in the Ziegler–Nichols calculation
    pub tu: f64,
}

/// State of the auto-tuning process
#[derive(Debug, Clone, Copy, PartialEq)]
enum AutoTuneState {
    NotStarted,
    Running,
    Completed,
    Failed,
}

/// Generic PID auto-tuner implementation
///
/// # Usage
/// ```rust,ignore
/// let config = AutoTuneConfig {
///     tune_delta: 0.5,   // ±0.5 bar
///     max_power: 1.0,
///     max_duration_secs: 600.0,
/// };
/// let mut tuner = PidAutoTuner::new(config);
/// let now = std::time::Instant::now();
/// tuner.start(now, 5.0);  // target = 5 bar
///
/// // In the control loop:
/// let (duty_cycle, done) = tuner.update(measured_value, now);
/// // map duty_cycle → actuator output
/// if done {
///     if let Some(result) = &tuner.result {
///         // apply result.kp / result.ki / result.kd
///     }
/// }
/// ```
#[derive(Debug)]
pub struct PidAutoTuner {
    config: AutoTuneConfig,
    state: AutoTuneState,

    /// Target setpoint (e.g. bar or °C)
    target_value: f64,

    /// `true` while the output is driven to `max_power` (increasing phase)
    pub heating: bool,

    // Peak tracking
    peak_value: f64,
    peak_time: Instant,
    peaks: Vec<(f64, Instant)>,

    // Timing
    start_time: Option<Instant>,

    /// Computed tuning result (available after `is_completed()` returns `true`)
    pub result: Option<AutoTuneResult>,
}

impl PidAutoTuner {
    /// Create a new PID auto-tuner with the given configuration
    pub fn new(config: AutoTuneConfig) -> Self {
        Self {
            config,
            state: AutoTuneState::NotStarted,
            target_value: 0.0,
            heating: false,
            peak_value: 0.0,
            peak_time: Instant::now(),
            peaks: Vec::new(),
            start_time: None,
            result: None,
        }
    }

    /// Start the auto-tuning process
    ///
    /// # Arguments
    /// * `now`          – current timestamp
    /// * `target_value` – the setpoint to oscillate around (bar, °C, …)
    pub fn start(&mut self, now: Instant, target_value: f64) {
        self.state = AutoTuneState::Running;
        self.target_value = target_value;
        self.heating = true;
        self.peak_value = f64::MAX; // when driving up, track the minimum (valley)
        self.peak_time = now;
        self.peaks.clear();
        self.start_time = Some(now);
        self.result = None;
    }

    /// Abort the auto-tuning process (marks state as `Failed`)
    pub fn stop(&mut self) {
        if self.state == AutoTuneState::Running {
            self.state = AutoTuneState::Failed;
        }
    }

    /// Feed the current measurement and advance the state machine.
    ///
    /// Returns `(duty_cycle, finished)`:
    /// * `duty_cycle` – 0.0 or `config.max_power`; map this to your actuator
    /// * `finished`   – `true` when the run is done (completed **or** failed)
    pub fn update(&mut self, current_value: f64, now: Instant) -> (f64, bool) {
        if self.state != AutoTuneState::Running {
            return (0.0, true);
        }

        // Check timeout
        if let Some(start) = self.start_time {
            if now.duration_since(start).as_secs_f64() > self.config.max_duration_secs {
                self.state = AutoTuneState::Failed;
                return (0.0, true);
            }
        }

        let upper_target = self.target_value + self.config.tune_delta;
        let lower_target = self.target_value - self.config.tune_delta;

        // Track peak values BEFORE checking for crossing
        if self.heating {
            // Driving up → we want to capture the minimum (valley)
            if current_value < self.peak_value {
                self.peak_value = current_value;
                self.peak_time = now;
            }
        } else {
            // Driving down → we want to capture the maximum (peak)
            if current_value > self.peak_value {
                self.peak_value = current_value;
                self.peak_time = now;
            }
        }

        // Check if we've crossed a boundary and should switch direction
        if self.heating && current_value >= upper_target {
            self.heating = false;
            self.record_peak();
        } else if !self.heating && current_value <= lower_target {
            self.heating = true;
            self.record_peak();
        }

        // Check if we have enough peaks to calculate PID parameters
        if self.peaks.len() >= 12 {
            self.calculate_pid();
            if self.state == AutoTuneState::Completed {
                return (0.0, true);
            }
        }

        let duty_cycle = if self.heating {
            self.config.max_power
        } else {
            0.0
        };

        (duty_cycle, false)
    }

    /// Record the current tracked peak and reset tracking for the next half-cycle
    fn record_peak(&mut self) {
        self.peaks.push((self.peak_value, self.peak_time));

        // Reset peak tracking for next cycle (following Klipper's logic)
        if self.heating {
            self.peak_value = f64::MAX; // driving up → find valley
        } else {
            self.peak_value = f64::MIN; // driving down → find peak
        }
    }

    /// Calculate PID parameters from the recorded oscillation peaks
    fn calculate_pid(&mut self) {
        if self.peaks.len() < 4 {
            self.state = AutoTuneState::Failed;
            return;
        }

        // Collect cycle times for all peaks starting at index 4
        let mut cycle_times: Vec<(f64, usize)> = Vec::new();
        for pos in 4..self.peaks.len() {
            let time_diff = self.peaks[pos]
                .1
                .duration_since(self.peaks[pos - 2].1)
                .as_secs_f64();
            cycle_times.push((time_diff, pos));
        }

        // Sort by time and pick the median cycle
        cycle_times.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let midpoint_pos = if cycle_times.is_empty() {
            self.state = AutoTuneState::Failed;
            return;
        } else {
            cycle_times[cycle_times.len() / 2].1
        };

        if let Some(result) = self.calculate_pid_at_position(midpoint_pos) {
            self.result = Some(result);
            self.state = AutoTuneState::Completed;
        } else {
            self.state = AutoTuneState::Failed;
        }
    }

    /// Calculate PID parameters at a specific peak position using the
    /// Astrom-Hagglund relay method and Ziegler-Nichols rules
    fn calculate_pid_at_position(&self, pos: usize) -> Option<AutoTuneResult> {
        if pos < 2 || pos >= self.peaks.len() {
            return None;
        }

        let value_diff = self.peaks[pos].0 - self.peaks[pos - 1].0;
        let time_diff = self.peaks[pos]
            .1
            .duration_since(self.peaks[pos - 2].1)
            .as_secs_f64();

        // Astrom-Hagglund: estimate ultimate gain (Ku) and ultimate period (Tu)
        let amplitude = 0.5 * value_diff.abs();
        if amplitude == 0.0 || time_diff == 0.0 {
            return None;
        }
        let ku = 4.0 * self.config.max_power / (std::f64::consts::PI * amplitude);
        let tu = time_diff;

        // Ziegler-Nichols rules
        let ti = 0.5 * tu;
        let td = 0.125 * tu;
        let kp = 0.4 * ku;
        let ki = kp / ti;
        let kd = kp * td;

        Some(AutoTuneResult { kp, ki, kd, ku, tu })
    }

    /// Returns `true` when auto-tuning completed successfully
    pub fn is_completed(&self) -> bool {
        self.state == AutoTuneState::Completed
    }

    /// Returns `true` when auto-tuning failed (timeout or insufficient data)
    pub fn is_failed(&self) -> bool {
        self.state == AutoTuneState::Failed
    }

    /// Returns `true` when auto-tuning is currently running
    pub fn is_running(&self) -> bool {
        self.state == AutoTuneState::Running
    }

    /// Current state as a static string slice
    pub fn state(&self) -> &str {
        match self.state {
            AutoTuneState::NotStarted => "not_started",
            AutoTuneState::Running => "running",
            AutoTuneState::Completed => "completed",
            AutoTuneState::Failed => "failed",
        }
    }

    /// Progress as a percentage in the range 0 – 100
    pub fn get_progress_percent(&self) -> f64 {
        match self.state {
            AutoTuneState::Completed => 100.0,
            AutoTuneState::NotStarted | AutoTuneState::Failed => 0.0,
            AutoTuneState::Running => {
                // Uses 12 peaks as the completion threshold
                let progress = (self.peaks.len() as f64 / 12.0) * 100.0;
                progress.min(99.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_autotuner_creation() {
        let config = AutoTuneConfig::default();
        let tuner = PidAutoTuner::new(config);
        assert_eq!(tuner.state(), "not_started");
        assert_eq!(tuner.get_progress_percent(), 0.0);
    }

    #[test]
    fn test_autotuner_start() {
        let config = AutoTuneConfig::default();
        let mut tuner = PidAutoTuner::new(config);
        let now = Instant::now();

        tuner.start(now, 150.0);
        assert_eq!(tuner.state(), "running");
        assert!(tuner.is_running());
        assert_eq!(tuner.target_value, 150.0);
    }

    #[test]
    fn test_autotuner_stop() {
        let config = AutoTuneConfig::default();
        let mut tuner = PidAutoTuner::new(config);
        let now = Instant::now();

        tuner.start(now, 150.0);
        tuner.stop();
        assert!(tuner.is_failed());
    }

    #[test]
    fn test_autotuner_oscillation() {
        let config = AutoTuneConfig {
            tune_delta: 5.0,
            max_power: 1.0,
            max_duration_secs: 3600.0,
        };
        let mut tuner = PidAutoTuner::new(config);
        let mut now = Instant::now();

        tuner.start(now, 150.0);

        let mut temp = 140.0_f64;
        let mut completed = false;
        let mut iterations = 0;

        for cycle in 0..20 {
            // Heating phase
            while temp < 155.0 && !completed && iterations < 1000 {
                temp += 0.5;
                now += Duration::from_millis(100);
                let (_duty_cycle, done) = tuner.update(temp, now);
                completed = done;
                iterations += 1;
                if !completed && !tuner.heating {
                    break;
                }
                if completed || temp >= 155.0 {
                    break;
                }
            }

            if completed {
                break;
            }

            // Cooling phase
            while temp > 145.0 && !completed && iterations < 1000 {
                temp -= 0.3;
                now += Duration::from_millis(100);
                let (_duty_cycle, done) = tuner.update(temp, now);
                completed = done;
                iterations += 1;
                if !completed && tuner.heating {
                    break;
                }
                if completed || temp <= 145.0 {
                    break;
                }
            }

            if completed {
                break;
            }

            if cycle % 3 == 0 {
                let progress = tuner.get_progress_percent();
                println!("Cycle {}: Progress {}%", cycle, progress);
            }
        }

        assert!(completed, "Auto-tuning should complete after sufficient oscillations");
        assert!(tuner.is_completed(), "Should be in completed state");
        assert!(tuner.result.is_some(), "Should have a result");

        let result = tuner.result.as_ref().unwrap();
        println!(
            "Auto-tune result: Kp={:.3}, Ki={:.3}, Kd={:.3}",
            result.kp, result.ki, result.kd
        );
        println!("Ku={:.3}, Tu={:.3}", result.ku, result.tu);

        assert!(result.kp > 0.0, "Kp should be positive");
        assert!(result.ki > 0.0, "Ki should be positive");
        assert!(result.kd > 0.0, "Kd should be positive");
    }

    #[test]
    fn test_autotuner_timeout() {
        let config = AutoTuneConfig {
            tune_delta: 5.0,
            max_power: 1.0,
            max_duration_secs: 1.0, // Very short timeout
        };
        let mut tuner = PidAutoTuner::new(config);
        let mut now = Instant::now();

        tuner.start(now, 150.0);

        now += Duration::from_secs(2);
        let (_, completed) = tuner.update(150.0, now);

        assert!(completed, "Should timeout");
        assert!(tuner.is_failed(), "Should fail on timeout");
    }

    /// Same as the oscillation test but with pressure-like values
    #[test]
    fn test_autotuner_pressure_oscillation() {
        let config = AutoTuneConfig {
            tune_delta: 0.5,  // ±0.5 bar
            max_power: 1.0,
            max_duration_secs: 3600.0,
        };
        let mut tuner = PidAutoTuner::new(config);
        let mut now = Instant::now();

        tuner.start(now, 5.0); // target 5 bar

        let mut pressure = 4.0_f64;
        let mut completed = false;
        let mut iterations = 0;

        for cycle in 0..20 {
            while pressure < 5.5 && !completed && iterations < 1000 {
                pressure += 0.05;
                now += Duration::from_millis(100);
                let (_duty_cycle, done) = tuner.update(pressure, now);
                completed = done;
                iterations += 1;
                if !completed && !tuner.heating {
                    break;
                }
                if completed || pressure >= 5.5 {
                    break;
                }
            }

            if completed {
                break;
            }

            while pressure > 4.5 && !completed && iterations < 1000 {
                pressure -= 0.03;
                now += Duration::from_millis(100);
                let (_duty_cycle, done) = tuner.update(pressure, now);
                completed = done;
                iterations += 1;
                if !completed && tuner.heating {
                    break;
                }
                if completed || pressure <= 4.5 {
                    break;
                }
            }

            if completed {
                break;
            }

            if cycle % 3 == 0 {
                println!("Pressure cycle {}: progress {}%", cycle, tuner.get_progress_percent());
            }
        }

        assert!(completed, "Pressure auto-tuning should complete");
        assert!(tuner.is_completed());
        let result = tuner.result.as_ref().unwrap();
        println!(
            "Pressure PID: Kp={:.4}, Ki={:.4}, Kd={:.4}",
            result.kp, result.ki, result.kd
        );
        assert!(result.kp > 0.0);
        assert!(result.ki > 0.0);
        assert!(result.kd > 0.0);
    }
}
