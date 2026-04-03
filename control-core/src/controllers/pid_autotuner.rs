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
//! returned duty-cycle (0.0 … `max_power`) to the actual actuator output.

use std::time::{Duration, Instant};

const REQUIRED_PEAK_COUNT: usize = 20;
const MIN_PEAKS_FOR_PID: usize = 4;
const MIN_PEAKS_FOR_CYCLE_SELECTION: usize = 5;

/// Configuration for PID auto-tuning
#[derive(Debug, Clone)]
pub struct AutoTuneConfig {
    /// Oscillation amplitude around the setpoint (e.g. ±5 °C or ±0.5 bar)
    pub tune_delta: f64,
    /// Maximum output fraction (0.0 … 1.0).  Maps to "full on" in bang-bang
    /// control during the tuning phase.
    pub max_power: f64,
    /// Maximum duration for auto-tuning before the run is aborted
    pub max_duration: Duration,
}

impl Default for AutoTuneConfig {
    fn default() -> Self {
        Self {
            tune_delta: 5.0,
            max_power: 1.0,
            max_duration: Duration::from_secs(3600),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoTuneError {
    NotStarted,
    Running,
    Failed,
}

/// Generic PID auto-tuner implementation
///
/// # Usage
/// ```rust,ignore
/// let config = AutoTuneConfig {
///     tune_delta: 0.5,   // ±0.5 bar
///     max_power: 1.0,
///     max_duration: std::time::Duration::from_secs(600),
/// };
/// let mut tuner = PidAutoTuner::new(config);
/// let now = std::time::Instant::now();
/// tuner.start(now, 5.0);  // target = 5 bar
///
/// // In the control loop:
/// let duty_cycle = tuner.update(measured_value, now);
/// // map duty_cycle → actuator output
/// if let Ok(result) = tuner.result() {
///     // apply result.kp / result.ki / result.kd
/// }
/// ```
#[derive(Debug)]
pub struct PidAutoTuner {
    config: AutoTuneConfig,
    state: AutoTuneState,

    /// Target setpoint (e.g. bar or °C)
    target_value: f64,

    /// `true` while the output is driven to `max_power`
    driving_high_output: bool,

    // Peak tracking (contains both peaks and valleys)
    tracked_peak_value: f64,
    tracked_peak_time: Instant,
    peaks: Vec<(f64, Instant)>,

    // Timing
    start_time: Option<Instant>,

    result: Option<AutoTuneResult>,
}

impl PidAutoTuner {
    /// Create a new PID auto-tuner with the given configuration
    #[must_use]
    pub fn new(config: AutoTuneConfig) -> Self {
        Self {
            config,
            state: AutoTuneState::NotStarted,
            target_value: 0.0,
            driving_high_output: false,
            tracked_peak_value: 0.0,
            tracked_peak_time: Instant::now(),
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
        self.driving_high_output = true;
        self.tracked_peak_value = f64::MAX; // while driving high, track the minimum
        self.tracked_peak_time = now;
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
    /// Returns the actuator output command (`0.0` or `config.max_power`).
    pub fn update(&mut self, current_value: f64, now: Instant) -> f64 {
        if self.state != AutoTuneState::Running {
            return 0.0;
        }

        // Check timeout
        if let Some(start) = self.start_time {
            if now.duration_since(start) > self.config.max_duration {
                self.state = AutoTuneState::Failed;
                return 0.0;
            }
        }

        let upper_target = self.target_value + self.config.tune_delta;
        let lower_target = self.target_value - self.config.tune_delta;

        // Track peak values BEFORE checking for crossing
        if self.driving_high_output {
            // Driving up → we want to capture the minimum (valley)
            if current_value < self.tracked_peak_value {
                self.tracked_peak_value = current_value;
                self.tracked_peak_time = now;
            }
        } else {
            // Driving down → we want to capture the maximum (peak)
            if current_value > self.tracked_peak_value {
                self.tracked_peak_value = current_value;
                self.tracked_peak_time = now;
            }
        }

        // Check if we've crossed a boundary and should switch direction
        if self.driving_high_output && current_value >= upper_target {
            self.driving_high_output = false;
            self.record_peak();
        } else if !self.driving_high_output && current_value <= lower_target {
            self.driving_high_output = true;
            self.record_peak();
        }

        // Check if we have enough peaks to calculate PID parameters
        if self.peaks.len() >= REQUIRED_PEAK_COUNT {
            self.calculate_pid();
            if self.state == AutoTuneState::Completed {
                return 0.0;
            }
        }

        if self.driving_high_output {
            self.config.max_power
        } else {
            0.0
        }
    }

    /// Record the current tracked peak and reset tracking for the next half-cycle
    fn record_peak(&mut self) {
        self.peaks
            .push((self.tracked_peak_value, self.tracked_peak_time));

        // Reset peak tracking for next cycle (following Klipper's logic)
        if self.driving_high_output {
            self.tracked_peak_value = f64::MAX; // driving high → find valley
        } else {
            self.tracked_peak_value = f64::MIN; // driving low → find peak
        }
    }

    /// Calculate PID parameters from the recorded oscillation peaks
    fn calculate_pid(&mut self) {
        if self.peaks.len() < MIN_PEAKS_FOR_PID {
            self.state = AutoTuneState::Failed;
            return;
        }

        if self.peaks.len() < MIN_PEAKS_FOR_CYCLE_SELECTION {
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

        if cycle_times.is_empty() {
            self.state = AutoTuneState::Failed;
            return;
        }

        // Sort by time and pick the median cycle
        cycle_times.sort_by(|a, b| a.0.total_cmp(&b.0));
        let midpoint_pos = cycle_times[cycle_times.len() / 2].1;

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
        let ti = 1.0 * tu;
        let td = 0.125 * tu;
        let kp = 0.25 * ku;
        let ki = kp / ti;
        let kd = kp * td;

        Some(AutoTuneResult { kp, ki, kd, ku, tu })
    }

    /// Returns `true` when auto-tuning completed successfully
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.state == AutoTuneState::Completed
    }

    /// Returns `true` when auto-tuning failed (timeout or insufficient data)
    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.state == AutoTuneState::Failed
    }

    /// Returns `true` when auto-tuning is currently running
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.state == AutoTuneState::Running
    }

    /// Returns `true` while the tuner drives the actuator at high output.
    #[must_use]
    pub const fn is_driving_high_output(&self) -> bool {
        self.driving_high_output
    }

    /// Current state as a static string slice
    #[must_use]
    pub const fn state(&self) -> &str {
        match self.state {
            AutoTuneState::NotStarted => "not_started",
            AutoTuneState::Running => "running",
            AutoTuneState::Completed => "completed",
            AutoTuneState::Failed => "failed",
        }
    }

    pub fn result(&self) -> Result<&AutoTuneResult, AutoTuneError> {
        match self.state {
            AutoTuneState::Completed => self.result.as_ref().ok_or(AutoTuneError::Failed),
            AutoTuneState::Failed => Err(AutoTuneError::Failed),
            AutoTuneState::Running => Err(AutoTuneError::Running),
            AutoTuneState::NotStarted => Err(AutoTuneError::NotStarted),
        }
    }

    /// Progress as a percentage in the range 0 – 100
    #[must_use]
    pub fn get_progress_percent(&self) -> f64 {
        match self.state {
            AutoTuneState::Completed => 100.0,
            AutoTuneState::NotStarted | AutoTuneState::Failed => 0.0,
            AutoTuneState::Running => {
                // Uses 20 peaks (10 full oscillation cycles) as the completion threshold
                let progress = (self.peaks.len() as f64 / REQUIRED_PEAK_COUNT as f64) * 100.0;
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
            max_duration: Duration::from_secs(3600),
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
                let _duty_cycle = tuner.update(temp, now);
                completed = tuner.is_completed() || tuner.is_failed();
                iterations += 1;
                if !completed && !tuner.is_driving_high_output() {
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
                let _duty_cycle = tuner.update(temp, now);
                completed = tuner.is_completed() || tuner.is_failed();
                iterations += 1;
                if !completed && tuner.is_driving_high_output() {
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

        assert!(
            completed,
            "Auto-tuning should complete after sufficient oscillations"
        );
        assert!(tuner.is_completed(), "Should be in completed state");
        assert!(tuner.result().is_ok(), "Should have a result");

        let result = tuner.result().unwrap();
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
            max_duration: Duration::from_secs(1),
        };
        let mut tuner = PidAutoTuner::new(config);
        let mut now = Instant::now();

        tuner.start(now, 150.0);

        now += Duration::from_secs(2);
        let _ = tuner.update(150.0, now);

        assert!(tuner.is_failed(), "Should fail on timeout");
    }

    /// Same as the oscillation test but with pressure-like values
    #[test]
    fn test_autotuner_pressure_oscillation() {
        let config = AutoTuneConfig {
            tune_delta: 0.5, // ±0.5 bar
            max_power: 1.0,
            max_duration: Duration::from_secs(3600),
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
                let _duty_cycle = tuner.update(pressure, now);
                completed = tuner.is_completed() || tuner.is_failed();
                iterations += 1;
                if !completed && !tuner.is_driving_high_output() {
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
                let _duty_cycle = tuner.update(pressure, now);
                completed = tuner.is_completed() || tuner.is_failed();
                iterations += 1;
                if !completed && tuner.is_driving_high_output() {
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
                println!(
                    "Pressure cycle {}: progress {}%",
                    cycle,
                    tuner.get_progress_percent()
                );
            }
        }

        assert!(completed, "Pressure auto-tuning should complete");
        assert!(tuner.is_completed());
        let result = tuner.result().unwrap();
        println!(
            "Pressure PID: Kp={:.4}, Ki={:.4}, Kd={:.4}",
            result.kp, result.ki, result.kd
        );
        assert!(result.kp > 0.0);
        assert!(result.ki > 0.0);
        assert!(result.kd > 0.0);
    }
}
