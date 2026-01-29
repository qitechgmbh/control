use std::time::Instant;

/// PID auto-tuning using Relay Method (Åström-Hägglund)
/// This is a robust and practical method for industrial applications
#[derive(Debug, Clone)]
pub struct PidAutoTuner {
    state: AutoTuneState,
    config: AutoTuneConfig,
    
    // Measurement data
    oscillation_data: Vec<OscillationPoint>,
    start_time: Option<Instant>,
    
    // Relay state tracking
    last_relay_high: bool,
    target_reached_once: bool,
    initial_error: Option<f64>,
    
    // Oscillation cycle counting
    completed_cycles: usize,
    last_peak_time: Option<f64>,
    prev_peak_time: Option<f64>,
    last_peak_value: Option<f64>,
    last_valley_time: Option<f64>,
    last_valley_value: Option<f64>,
    cycle_samples: Vec<CycleSample>,

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
    /// Target temperature/setpoint for the process
    pub target: f64,
    /// Hysteresis for relay switching (oscillation will be ±hysteresis around target)
    pub hysteresis: f64,
    /// Relay output amplitude (0.0 - 1.0)
    pub relay_amplitude: f64,
    /// Output used for the initial approach to target
    pub approach_output: f64,
    /// Minimum number of oscillations to observe
    pub min_oscillations: usize,
    /// Minimum time between peaks (seconds) to filter noise
    pub min_cycle_secs: f64,
    /// Maximum time to run auto-tuning before giving up
    pub max_duration_secs: f64,
}

impl Default for AutoTuneConfig {
    fn default() -> Self {
        Self {
            target: 150.0,
            hysteresis: 5.0, // ±5°C oscillation around target
            relay_amplitude: 1.0,
            approach_output: 1.0,
            min_oscillations: 5, // 5 complete cycles for accurate tuning
            min_cycle_secs: 4.0,
            max_duration_secs: 2400.0, // 40 minutes
        }
    }
}

#[derive(Debug, Clone)]
struct OscillationPoint {
    time: f64,
    value: f64,
    is_peak: bool,
}

#[derive(Debug, Clone)]
pub struct AutoTuneResult {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub ultimate_gain: f64,
    pub ultimate_period: f64,
}

#[derive(Debug, Clone)]
struct CycleSample {
    period: f64,
    amplitude: f64,
}

impl PidAutoTuner {
    pub fn new(config: AutoTuneConfig) -> Self {
        Self {
            state: AutoTuneState::Idle,
            config,
            oscillation_data: Vec::new(),
            start_time: None,
            last_relay_high: false,
            target_reached_once: false,
            initial_error: None,
            completed_cycles: 0,
            last_peak_time: None,
            prev_peak_time: None,
            last_peak_value: None,
            last_valley_time: None,
            last_valley_value: None,
            cycle_samples: Vec::new(),
            progress_percent: 0.0,
            result: None,
        }
    }

    pub fn start(&mut self, now: Instant) {
        self.state = AutoTuneState::MeasuringOscillations;
        self.oscillation_data.clear();
        self.start_time = Some(now);
        self.last_relay_high = true; // Start with heating ON
        self.target_reached_once = false;
        self.initial_error = None;
        self.completed_cycles = 0;
        self.last_peak_time = None;
        self.prev_peak_time = None;
        self.last_peak_value = None;
        self.last_valley_time = None;
        self.last_valley_value = None;
        self.cycle_samples.clear();
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

    /// Get the number of completed oscillation cycles
    pub fn get_completed_cycles(&self) -> usize {
        self.completed_cycles
    }

    /// Get the required number of cycles
    pub fn get_required_cycles(&self) -> usize {
        self.config.min_oscillations
    }

    /// Update the auto-tuner with current process value and get control output
    /// Returns: (control_output, is_complete)
    pub fn update(&mut self, current_value: f64, now: Instant) -> (f64, bool) {
        let Some(start_time) = self.start_time else {
            return (0.0, false);
        };

        let elapsed = now.duration_since(start_time).as_secs_f64();

        // Check for timeout
        if elapsed > self.config.max_duration_secs {
            self.state = AutoTuneState::Failed("Timeout - auto-tuning took too long".to_string());
            return (0.0, true);
        }

        match self.state {
            AutoTuneState::MeasuringOscillations => {
                if self.initial_error.is_none() {
                    let initial_error = (self.config.target - current_value).abs().max(1.0);
                    self.initial_error = Some(initial_error);
                }

                // Record measurement
                self.oscillation_data.push(OscillationPoint {
                    time: elapsed,
                    value: current_value,
                    is_peak: false,
                });

                // Simple bang-bang relay control (like Arduino implementation)
                // Output is ON and input has risen to target -> turn OFF
                // Output is OFF and input has dropped to target -> turn ON
                let high_threshold = self.config.target + self.config.hysteresis;
                let low_threshold = self.config.target - self.config.hysteresis;

                let output = if self.last_relay_high && current_value > high_threshold {
                    // Heating was ON, now above target: turn OFF and mark target reached
                    // This is a PEAK - count it as half a cycle
                    self.last_relay_high = false;
                    self.record_peak(elapsed, current_value);
                    
                    if !self.target_reached_once {
                        self.target_reached_once = true;
                    } else {
                        self.update_cycle_count();
                    }
                    0.0
                } else if !self.last_relay_high && current_value < low_threshold {
                    // Heating was OFF, now below target: turn ON
                    self.last_relay_high = true;
                    self.record_valley(elapsed, current_value);
                    
                    // Use limited power until target reached once, then full relay amplitude
                    if self.target_reached_once {
                        self.config.relay_amplitude
                    } else {
                        self.config.approach_output
                    }
                } else {
                    // Maintain current state
                    if self.last_relay_high {
                        // Keep same power level based on whether target was reached
                        if self.target_reached_once {
                            self.config.relay_amplitude
                        } else {
                            self.config.approach_output
                        }
                    } else {
                        0.0
                    }
                };

                self.update_progress(current_value);

                // Check if we have enough cycles
                if self.should_finish() {
                    if let Some(result) = self.analyze_oscillations() {
                        self.result = Some(result);
                        self.state = AutoTuneState::Completed;
                        return (0.0, true);
                    }
                }

                (output, false)
            }
            AutoTuneState::Completed | AutoTuneState::Failed(_) | AutoTuneState::Idle => (0.0, true),
        }
    }

    fn analyze_oscillations(&self) -> Option<AutoTuneResult> {
        if self.cycle_samples.len() < 2 {
            return None;
        }

        let sample_count = self.cycle_samples.len();
        let window = sample_count.min(5);
        let window_samples = &self.cycle_samples[sample_count - window..];

        let ultimate_period = window_samples.iter().map(|s| s.period).sum::<f64>() / window as f64;
        let amplitude = window_samples.iter().map(|s| s.amplitude).sum::<f64>() / window as f64;

        if amplitude < 0.1 || ultimate_period <= 0.0 {
            // Oscillations too small
            return None;
        }

        // Calculate ultimate gain using relay method formula
        // Ku = 4d / (π * a)
        // where d is relay amplitude (0 to 1 in our case = 1.0) and a is process oscillation amplitude
        let relay_amplitude = self.config.relay_amplitude.max(0.01);
        let ultimate_gain = (4.0 * relay_amplitude) / (std::f64::consts::PI * amplitude);

        // Apply Tyreus-Luyben tuning rules - designed for processes with lag/delay like temperature control
        // Very conservative, minimizes overshoot, ideal for thermal systems with high inertia
        // Tyreus-Luyben: Kp = Ku/3.2, Ti = 2.2*Tu, Td = Tu/6.3
        let kp = ultimate_gain / 3.2;
        let ki = kp / (2.2 * ultimate_period);  // Ki = Kp/Ti
        let kd = kp * ultimate_period / 6.3;     // Kd = Kp*Td - very gentle derivative action

        Some(AutoTuneResult {
            kp,
            ki,
            kd,
            ultimate_gain,
            ultimate_period,
        })
    }

    fn record_peak(&mut self, time: f64, value: f64) {
        self.prev_peak_time = self.last_peak_time;
        self.last_peak_time = Some(time);
        self.last_peak_value = Some(value);
    }

    fn record_valley(&mut self, time: f64, value: f64) {
        self.last_valley_time = Some(time);
        self.last_valley_value = Some(value);
    }

    fn update_cycle_count(&mut self) {
        let (Some(peak_time), Some(peak_value)) = (self.last_peak_time, self.last_peak_value) else {
            return;
        };
        let (Some(prev_peak_time), Some(valley_time), Some(valley_value)) =
            (self.prev_peak_time, self.last_valley_time, self.last_valley_value)
        else {
            return;
        };

        if valley_time <= prev_peak_time || valley_time >= peak_time {
            // Valley must be between peaks
            return;
        }

        let period = peak_time - prev_peak_time;
        if period < self.config.min_cycle_secs {
            return;
        }

        let amplitude = (peak_value - valley_value).abs() / 2.0;
        if amplitude > 0.0 {
            self.cycle_samples.push(CycleSample { period, amplitude });
            self.completed_cycles = self.cycle_samples.len();
        }
    }

    fn update_progress(&mut self, current_value: f64) {
        if !self.target_reached_once {
            let Some(initial_error) = self.initial_error else {
                self.progress_percent = 0.0;
                return;
            };
            let error = (self.config.target - current_value).abs();
            let approach_progress = (1.0 - (error / initial_error).min(1.0)) * 20.0;
            self.progress_percent = approach_progress.clamp(0.0, 20.0);
        } else {
            let cycle_progress = self.completed_cycles as f64 / self.config.min_oscillations as f64;
            self.progress_percent = 20.0 + (cycle_progress * 80.0).min(80.0);
        }
    }

    fn should_finish(&self) -> bool {
        if self.completed_cycles >= self.config.min_oscillations {
            return true;
        }

        if self.cycle_samples.len() < 3 {
            return false;
        }

        let window = &self.cycle_samples[self.cycle_samples.len() - 3..];
        let mean_period = window.iter().map(|s| s.period).sum::<f64>() / 3.0;
        let mean_amp = window.iter().map(|s| s.amplitude).sum::<f64>() / 3.0;

        if mean_period <= 0.0 || mean_amp <= 0.0 {
            return false;
        }

        let max_period_dev = window
            .iter()
            .map(|s| ((s.period - mean_period).abs() / mean_period))
            .fold(0.0, f64::max);
        let max_amp_dev = window
            .iter()
            .map(|s| ((s.amplitude - mean_amp).abs() / mean_amp))
            .fold(0.0, f64::max);

        max_period_dev < 0.2 && max_amp_dev < 0.2
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
        tuner.start(Instant::now());
        assert_eq!(tuner.state, AutoTuneState::MeasuringOscillations);
        assert!(tuner.is_active());
    }

    #[test]
    fn test_autotuner_relay_output() {
        let config = AutoTuneConfig {
            relay_amplitude: 0.6,
            approach_output: 0.6,
            hysteresis: 0.0,
            ..Default::default()
        };
        let mut tuner = PidAutoTuner::new(config);
        let now = Instant::now();
        tuner.start(now);

        // After settle time, should be in measuring state
        let later = now + Duration::from_secs(1);
        
        // When below target, output should be high
        let (output, _) = tuner.update(100.0, later);
        assert!(output > 0.5);

        // When above target, output should be low
        let (output, _) = tuner.update(200.0, later + Duration::from_secs(1));
        assert!(output < 0.1);
    }
}
