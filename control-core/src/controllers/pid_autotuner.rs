//! PID Auto-tuner based on Klipper's algorithm
//!
//! This implementation uses the Astrom-Hagglund relay method to identify
//! system oscillations, then applies Ziegler-Nichols tuning rules to
//! calculate PID parameters.
//!
//! The algorithm works by:
//! 1. Heating to target + delta, then cooling to target - delta repeatedly
//! 2. Recording temperature peaks during these oscillations
//! 3. Calculating the ultimate gain (Ku) and period (Tu) from peak data
//! 4. Applying Ziegler-Nichols rules to derive Kp, Ki, Kd values

use std::time::Instant;

/// Configuration for PID auto-tuning
#[derive(Debug, Clone)]
pub struct AutoTuneConfig {
    /// Temperature oscillation amplitude (e.g., ±5°C)
    pub tune_delta: f64,
    /// Maximum heating power (0.0 to 1.0)
    pub max_power: f64,
    /// Maximum duration for auto-tuning in seconds
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
#[derive(Debug, Clone)]
pub struct AutoTuneResult {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    /// Ultimate gain (Ku) used in calculation
    pub ku: f64,
    /// Ultimate period (Tu) used in calculation
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

/// PID auto-tuner implementation
#[derive(Debug)]
pub struct PidAutoTuner {
    config: AutoTuneConfig,
    state: AutoTuneState,
    
    // Target temperature
    target_temp: f64,
    
    // Heating control
    heating: bool,
    
    // Peak tracking
    peak_value: f64,
    peak_time: Instant,
    peaks: Vec<(f64, Instant)>,
    
    // Timing
    start_time: Option<Instant>,
    
    // Results
    pub result: Option<AutoTuneResult>,
}

impl PidAutoTuner {
    /// Create a new PID auto-tuner with given configuration
    pub fn new(config: AutoTuneConfig) -> Self {
        Self {
            config,
            state: AutoTuneState::NotStarted,
            target_temp: 0.0,
            heating: false,
            peak_value: 0.0,
            peak_time: Instant::now(),
            peaks: Vec::new(),
            start_time: None,
            result: None,
        }
    }
    
    /// Start the auto-tuning process
    pub fn start(&mut self, now: Instant, target_temp: f64) {
        self.state = AutoTuneState::Running;
        self.target_temp = target_temp;
        self.heating = true;
        self.peak_value = 9999999.0; // When heating, track minimum (valley)
        self.peak_time = now;
        self.peaks.clear();
        self.start_time = Some(now);
        self.result = None;
    }
    
    /// Stop the auto-tuning process
    pub fn stop(&mut self) {
        if self.state == AutoTuneState::Running {
            self.state = AutoTuneState::Failed;
        }
    }
    
    /// Update the auto-tuner with current temperature
    /// Returns (duty_cycle, completed)
    pub fn update(&mut self, current_temp: f64, now: Instant) -> (f64, bool) {
        if self.state != AutoTuneState::Running {
            return (0.0, true);
        }
        
        // Check for timeout
        if let Some(start) = self.start_time {
            let elapsed = now.duration_since(start).as_secs_f64();
            if elapsed > self.config.max_duration_secs {
                self.state = AutoTuneState::Failed;
                return (0.0, true);
            }
        }
        
        // Temperature targets for oscillation
        let upper_target = self.target_temp + self.config.tune_delta;
        let lower_target = self.target_temp - self.config.tune_delta;
        
        // Track peak values BEFORE checking for crossing
        if self.heating {
            // When heating, track minimum (valley)
            if current_temp < self.peak_value {
                self.peak_value = current_temp;
                self.peak_time = now;
            }
        } else {
            // When cooling, track maximum (peak)
            if current_temp > self.peak_value {
                self.peak_value = current_temp;
                self.peak_time = now;
            }
        }
        
        // Check if we've crossed the target and need to switch heating/cooling
        if self.heating && current_temp >= upper_target {
            self.heating = false;
            self.record_peak();
        } else if !self.heating && current_temp <= lower_target {
            self.heating = true;
            self.record_peak();
        }
        
        // Check if we have enough peaks to calculate PID
        if self.peaks.len() >= 12 {
            self.calculate_pid();
            self.state = AutoTuneState::Completed;
            return (0.0, true);
        }
        
        // Return appropriate duty cycle
        let duty_cycle = if self.heating {
            self.config.max_power
        } else {
            0.0
        };
        
        (duty_cycle, false)
    }
    
    /// Record a peak in the oscillation
    fn record_peak(&mut self) {
        self.peaks.push((self.peak_value, self.peak_time));
        
        // Reset peak tracking for next cycle (following Klipper's logic)
        if self.heating {
            self.peak_value = 9999999.0; // When heating, we want to find the minimum (valley)
        } else {
            self.peak_value = -9999999.0;  // When cooling, we want to find the maximum (peak)
        }
    }
    
    /// Calculate PID parameters from recorded peaks
    fn calculate_pid(&mut self) {
        if self.peaks.len() < 4 {
            self.state = AutoTuneState::Failed;
            return;
        }
        
        // Find the cycle times and select the median cycle
        let mut cycle_times: Vec<(f64, usize)> = Vec::new();
        for pos in 4..self.peaks.len() {
            let time_diff = self.peaks[pos].1
                .duration_since(self.peaks[pos - 2].1)
                .as_secs_f64();
            cycle_times.push((time_diff, pos));
        }
        
        // Sort by time and pick the median
        cycle_times.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let midpoint_pos = if cycle_times.is_empty() {
            self.state = AutoTuneState::Failed;
            return;
        } else {
            cycle_times[cycle_times.len() / 2].1
        };
        
        // Calculate PID parameters using this cycle
        if let Some(result) = self.calculate_pid_at_position(midpoint_pos) {
            self.result = Some(result);
            self.state = AutoTuneState::Completed;
        } else {
            self.state = AutoTuneState::Failed;
        }
    }
    
    /// Calculate PID at a specific peak position using Astrom-Hagglund and Ziegler-Nichols
    fn calculate_pid_at_position(&self, pos: usize) -> Option<AutoTuneResult> {
        if pos < 2 || pos >= self.peaks.len() {
            return None;
        }
        
        // Calculate temperature amplitude and period
        let temp_diff = self.peaks[pos].0 - self.peaks[pos - 1].0;
        let time_diff = self.peaks[pos].1
            .duration_since(self.peaks[pos - 2].1)
            .as_secs_f64();
        
        // Astrom-Hagglund method to estimate Ku and Tu
        let amplitude = 0.5 * temp_diff.abs();
        let ku = 4.0 * self.config.max_power / (std::f64::consts::PI * amplitude);
        let tu = time_diff;
        
        // Ziegler-Nichols method to generate PID parameters
        let ti = 0.5 * tu;
        let td = 0.125 * tu;
        let kp = 0.6 * ku;
        let ki = kp / ti;
        let kd = kp * td;
        
        Some(AutoTuneResult {
            kp,
            ki,
            kd,
            ku,
            tu,
        })
    }
    
    /// Check if auto-tuning is completed successfully
    pub fn is_completed(&self) -> bool {
        self.state == AutoTuneState::Completed
    }
    
    /// Check if auto-tuning failed
    pub fn is_failed(&self) -> bool {
        self.state == AutoTuneState::Failed
    }
    
    /// Check if auto-tuning is running
    pub fn is_running(&self) -> bool {
        self.state == AutoTuneState::Running
    }
    
    /// Get the current state
    pub fn state(&self) -> &str {
        match self.state {
            AutoTuneState::NotStarted => "not_started",
            AutoTuneState::Running => "running",
            AutoTuneState::Completed => "completed",
            AutoTuneState::Failed => "failed",
        }
    }
    
    /// Get progress as a percentage (0-100)
    pub fn get_progress_percent(&self) -> f64 {
        if self.state != AutoTuneState::Running {
            return if self.state == AutoTuneState::Completed {
                100.0
            } else {
                0.0
            };
        }
        
        // Progress is based on number of peaks collected (need 12)
        let progress = (self.peaks.len() as f64 / 12.0) * 100.0;
        progress.min(99.0) // Cap at 99% until actually completed
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
        assert_eq!(tuner.target_temp, 150.0);
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
            max_duration_secs: 3600.0, // 1 hour
        };
        let mut tuner = PidAutoTuner::new(config);
        let mut now = Instant::now();
        
        tuner.start(now, 150.0);
        
        // Simulate temperature oscillations
        // Should heat to 155, then cool to 145, repeatedly
        let mut temp = 140.0;
        let mut completed = false;
        let mut iterations = 0;
        
        for cycle in 0..20 {
            // Heating phase
            while temp < 155.0 && !completed && iterations < 1000 {
                temp += 0.5; // Simulate temperature rise
                now += Duration::from_millis(100);
                let (duty_cycle, done) = tuner.update(temp, now);
                completed = done;
                iterations += 1;
                
                if !completed && !tuner.heating {
                    // We switched to cooling during heating phase - this is expected near the boundary
                    println!("Switched to cooling at temp={}", temp);
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
                temp -= 0.3; // Simulate temperature drop
                now += Duration::from_millis(100);
                let (duty_cycle, done) = tuner.update(temp, now);
                completed = done;
                iterations += 1;
                
                if !completed && tuner.heating {
                    // We switched to heating during cooling phase - this is expected near the boundary
                    println!("Switched to heating at temp={}", temp);
                    break;
                }
                if completed || temp <= 145.0 {
                    break;
                }
            }
            
            if completed {
                break;
            }
            
            // Check progress
            if cycle % 3 == 0 {
                let progress = tuner.get_progress_percent();
                println!("Cycle {}: Progress {}%", cycle, progress);
            }
        }
        
        assert!(completed, "Auto-tuning should complete after sufficient oscillations");
        assert!(tuner.is_completed(), "Should be in completed state");
        assert!(tuner.result.is_some(), "Should have a result");
        
        let result = tuner.result.as_ref().unwrap();
        println!("Auto-tune result: Kp={:.3}, Ki={:.3}, Kd={:.3}", result.kp, result.ki, result.kd);
        println!("Ku={:.3}, Tu={:.3}", result.ku, result.tu);
        
        // Basic sanity checks
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
        
        // Wait for timeout
        now += Duration::from_secs(2);
        let (_, completed) = tuner.update(150.0, now);
        
        assert!(completed, "Should timeout");
        assert!(tuner.is_failed(), "Should fail on timeout");
    }
}
