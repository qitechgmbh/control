use std::time::Instant;

/// PID auto-tuning using Relay Method (Åström-Hägglund)
/// Implementation based on Arduino-PID-AutoTune-Library
#[derive(Debug, Clone)]
pub struct PidAutoTuner {
    state: AutoTuneState,
    config: AutoTuneConfig,
    
    // Measurement data
    start_time: Option<Instant>,
    last_time: Option<Instant>,
    
    // Relay state tracking
    setpoint: f64,
    output_start: f64,
    
    // Peak detection using lookback window
    last_inputs: Vec<f64>,
    n_lookback: usize,
    peak_type: i32, // 0=none, 1=max, -1=min
    peak_count: usize,
    peaks: Vec<f64>,
    peak1_time: Option<Instant>,
    peak2_time: Option<Instant>,
    just_changed: bool,
    just_evaluated: bool,
    
    // Range tracking
    abs_max: f64,
    abs_min: f64,
    
    // Progress tracking
    progress_percent: f64,
    
    // Results
    pub result: Option<AutoTuneResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AutoTuneState {
    Idle,
    MeasuringOscillations,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct AutoTuneConfig {
    /// Output step size - how far above and below the starting output value will the output step
    pub output_step: f64,
    /// Noise band - the autotune will ignore signal chatter smaller than this value
    pub noise_band: f64,
    /// How far back we look to identify peaks (in number of samples)
    pub lookback_samples: usize,
    /// Control type: 0=PI, 1=PID
    pub control_type: u8,
    /// Maximum time to run auto-tuning before giving up (seconds)
    pub max_duration_secs: f64,
    /// Sample time in milliseconds
    pub sample_time_ms: u64,
}

impl Default for AutoTuneConfig {
    fn default() -> Self {
        Self {
            output_step: 0.3, // 30% step above/below starting output
            noise_band: 0.5,
            lookback_samples: 40, // 10 seconds at 250ms sample time
            control_type: 1, // PID
            max_duration_secs: 2400.0, // 40 minutes
            sample_time_ms: 250,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutoTuneResult {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub ultimate_gain: f64,
    pub ultimate_period: f64,
}

impl PidAutoTuner {
    pub fn new(config: AutoTuneConfig) -> Self {
        let n_lookback = config.lookback_samples;
        Self {
            state: AutoTuneState::Idle,
            config,
            start_time: None,
            last_time: None,
            setpoint: 0.0,
            output_start: 0.0,
            last_inputs: vec![0.0; n_lookback + 1],
            n_lookback,
            peak_type: 0,
            peak_count: 0,
            peaks: vec![0.0; 10],
            peak1_time: None,
            peak2_time: None,
            just_changed: false,
            just_evaluated: false,
            abs_max: 0.0,
            abs_min: 0.0,
            progress_percent: 0.0,
            result: None,
        }
    }

    pub fn start(&mut self, now: Instant, current_output: f64) {
        self.state = AutoTuneState::MeasuringOscillations;
        self.start_time = Some(now);
        self.last_time = Some(now);
        self.output_start = current_output;
        self.peak_type = 0;
        self.peak_count = 0;
        self.just_changed = false;
        self.just_evaluated = false;
        self.abs_max = 0.0;
        self.abs_min = 0.0;
        self.peak1_time = None;
        self.peak2_time = None;
        self.last_inputs.fill(0.0);
        self.peaks.fill(0.0);
        self.progress_percent = 0.0;
        self.result = None;
    }

    pub fn stop(&mut self) {
        self.state = AutoTuneState::Idle;
        self.start_time = None;
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, AutoTuneState::MeasuringOscillations)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.state, AutoTuneState::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.state, AutoTuneState::Failed(_))
    }

    pub fn get_state(&self) -> &AutoTuneState {
        &self.state
    }

    /// Get the number of peaks detected
    pub fn get_completed_cycles(&self) -> usize {
        self.peak_count
    }

    /// Get the required number of peaks (10 in Arduino implementation)
    pub fn get_required_cycles(&self) -> usize {
        10
    }

    /// Update the auto-tuner with current process value and get control output
    /// Returns: (control_output, is_complete)
    pub fn update(&mut self, current_value: f64, now: Instant) -> (f64, bool) {
        self.just_evaluated = false;
        
        // Check if we've collected enough peaks
        if self.peak_count > 9 && self.is_active() {
            self.finish_up();
            self.state = AutoTuneState::Completed;
            return (self.output_start, true);
        }

        let Some(start_time) = self.start_time else {
            return (0.0, false);
        };

        // Check for timeout
        let elapsed = now.duration_since(start_time).as_secs_f64();
        if elapsed > self.config.max_duration_secs {
            self.state = AutoTuneState::Failed("Timeout - auto-tuning took too long".to_string());
            return (self.output_start, true);
        }

        // Check sample time
        if let Some(last_time) = self.last_time {
            let time_diff = now.duration_since(last_time).as_millis() as u64;
            if time_diff < self.config.sample_time_ms {
                return (self.get_output(current_value), false);
            }
        }
        
        self.last_time = Some(now);
        self.just_evaluated = true;

        match self.state {
            AutoTuneState::MeasuringOscillations => {
                // Initialize on first run
                if self.peak_type == 0 && self.peak_count == 0 && self.abs_max == 0.0 {
                    self.setpoint = current_value;
                    self.abs_max = current_value;
                    self.abs_min = current_value;
                }
                
                // Track min/max
                if current_value > self.abs_max {
                    self.abs_max = current_value;
                }
                if current_value < self.abs_min {
                    self.abs_min = current_value;
                }

                // Detect peaks using lookback window
                let mut is_max = true;
                let mut is_min = true;
                
                // Check if current value is max or min in lookback window
                for i in (0..self.n_lookback).rev() {
                    let val = self.last_inputs[i];
                    if is_max {
                        is_max = current_value > val;
                    }
                    if is_min {
                        is_min = current_value < val;
                    }
                    self.last_inputs[i + 1] = self.last_inputs[i];
                }
                self.last_inputs[0] = current_value;

                // We need to fill the lookback buffer before trusting peaks
                if self.just_evaluated {
                    // Only process peaks after buffer is filled (check done in peak detection logic)
                    if is_max {
                        if self.peak_type == 0 {
                            self.peak_type = 1;
                        }
                        if self.peak_type == -1 {
                            self.peak_type = 1;
                            self.just_changed = true;
                            self.peak2_time = self.peak1_time;
                        }
                        self.peak1_time = Some(now);
                        self.peaks[self.peak_count] = current_value;
                    } else if is_min {
                        if self.peak_type == 0 {
                            self.peak_type = -1;
                        }
                        if self.peak_type == 1 {
                            self.peak_type = -1;
                            self.peak_count += 1;
                            self.just_changed = true;
                        }
                        
                        if self.peak_count < 10 {
                            self.peaks[self.peak_count] = current_value;
                        }
                    }

                    // Check if oscillations have stabilized
                    if self.just_changed && self.peak_count > 2 {
                        let avg_separation = (self.peaks[self.peak_count - 1] - self.peaks[self.peak_count - 2]).abs()
                            + (self.peaks[self.peak_count - 2] - self.peaks[self.peak_count - 3]).abs();
                        let avg_separation = avg_separation / 2.0;
                        
                        if avg_separation < 0.05 * (self.abs_max - self.abs_min) {
                            self.finish_up();
                            self.state = AutoTuneState::Completed;
                            return (self.output_start, true);
                        }
                    }
                    
                    self.just_changed = false;
                }

                self.update_progress();

                (self.get_output(current_value), false)
            }
            AutoTuneState::Completed | AutoTuneState::Failed(_) | AutoTuneState::Idle => {
                (self.output_start, true)
            }
        }
    }

    fn get_output(&self, current_value: f64) -> f64 {
        // Oscillate the output based on the input's relation to the setpoint
        if current_value > self.setpoint + self.config.noise_band {
            self.output_start - self.config.output_step
        } else if current_value < self.setpoint - self.config.noise_band {
            self.output_start + self.config.output_step
        } else {
            self.output_start
        }
    }

    fn finish_up(&mut self) {
        // Calculate tuning parameters using Ziegler-Nichols rules
        
        // Ku = 4 * (2 * oStep) / ((absMax - absMin) * π)
        // Note: Arduino uses 2*oStep because peak-to-peak relay amplitude is 2*oStep
        let amplitude = self.abs_max - self.abs_min;
        if amplitude <= 0.0 {
            self.state = AutoTuneState::Failed("No oscillations detected".to_string());
            return;
        }
        
        let ultimate_gain = (4.0 * 2.0 * self.config.output_step) / (amplitude * std::f64::consts::PI);
        
        // Calculate period from peak times
        let ultimate_period = if let (Some(peak1), Some(peak2)) = (self.peak1_time, self.peak2_time) {
            peak1.duration_since(peak2).as_secs_f64()
        } else {
            self.state = AutoTuneState::Failed("Could not determine oscillation period".to_string());
            return;
        };

        if ultimate_period <= 0.0 {
            self.state = AutoTuneState::Failed("Invalid oscillation period".to_string());
            return;
        }

        // Ziegler-Nichols tuning rules
        let (kp, ki, kd) = if self.config.control_type == 1 {
            // PID: Kp = 0.6*Ku, Ki = 1.2*Ku/Pu, Kd = 0.075*Ku*Pu
            let kp = 0.6 * ultimate_gain;
            let ki = 1.2 * ultimate_gain / ultimate_period;
            let kd = 0.075 * ultimate_gain * ultimate_period;
            (kp, ki, kd)
        } else {
            // PI: Kp = 0.4*Ku, Ki = 0.48*Ku/Pu, Kd = 0
            let kp = 0.4 * ultimate_gain;
            let ki = 0.48 * ultimate_gain / ultimate_period;
            let kd = 0.0;
            (kp, ki, kd)
        };

        self.result = Some(AutoTuneResult {
            kp,
            ki,
            kd,
            ultimate_gain,
            ultimate_period,
        });
    }

    fn update_progress(&mut self) {
        if self.peak_count == 0 {
            self.progress_percent = 10.0;
        } else {
            let progress = (self.peak_count as f64 / 10.0) * 90.0;
            self.progress_percent = 10.0 + progress.min(90.0);
        }
    }

    pub fn get_progress_percent(&self) -> f64 {
        match self.state {
            AutoTuneState::Idle => 0.0,
            AutoTuneState::MeasuringOscillations => {
                self.progress_percent
            }
            AutoTuneState::Completed => 100.0,
            AutoTuneState::Failed(_) => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_autotuner_initialization() {
        let config = AutoTuneConfig::default();
        let tuner = PidAutoTuner::new(config);
        assert_eq!(tuner.state, AutoTuneState::Idle);
        assert!(!tuner.is_active());
    }

    #[test]
    fn test_autotuner_start() {
        let config = AutoTuneConfig::default();
        let mut tuner = PidAutoTuner::new(config);
        let now = Instant::now();
        tuner.start(now, 0.5);
        assert_eq!(tuner.state, AutoTuneState::MeasuringOscillations);
        assert!(tuner.is_active());
    }

    #[test]
    fn test_autotuner_relay_output() {
        let config = AutoTuneConfig {
            output_step: 0.3,
            noise_band: 1.0,
            ..Default::default()
        };
        let mut tuner = PidAutoTuner::new(config);
        let now = Instant::now();
        let starting_output = 0.5;
        tuner.start(now, starting_output);

        // Initialize with a value
        let later = now + Duration::from_millis(300);
        let (_output, _) = tuner.update(100.0, later);
        
        // When below setpoint, output should be high
        let later = later + Duration::from_millis(300);
        let (output, _) = tuner.update(98.0, later);
        assert!((output - (starting_output + 0.3)).abs() < 0.01);

        // When above setpoint, output should be low
        let later = later + Duration::from_millis(300);
        let (output, _) = tuner.update(102.0, later);
        assert!((output - (starting_output - 0.3)).abs() < 0.01);
    }
}
