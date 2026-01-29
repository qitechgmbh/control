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
    /// Minimum number of oscillations to observe
    pub min_oscillations: usize,
    /// Maximum time to run auto-tuning before giving up
    pub max_duration_secs: f64,
}

impl Default for AutoTuneConfig {
    fn default() -> Self {
        Self {
            target: 150.0,
            hysteresis: 5.0, // ±5°C oscillation around target
            min_oscillations: 3,
            max_duration_secs: 600.0, // 10 minutes
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

impl PidAutoTuner {
    pub fn new(config: AutoTuneConfig) -> Self {
        Self {
            state: AutoTuneState::Idle,
            config,
            oscillation_data: Vec::new(),
            start_time: None,
            last_relay_high: false,
            result: None,
        }
    }

    pub fn start(&mut self, now: Instant) {
        self.state = AutoTuneState::MeasuringOscillations;
        self.oscillation_data.clear();
        self.start_time = Some(now);
        self.last_relay_high = true; // Start with heating ON
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
                // Record measurement
                self.oscillation_data.push(OscillationPoint {
                    time: elapsed,
                    value: current_value,
                    is_peak: false,
                });

                // Simple bang-bang relay control (like Arduino implementation)
                // Output is ON and input has risen to target -> turn OFF
                // Output is OFF and input has dropped to target -> turn ON
                let output = if self.last_relay_high && current_value > self.config.target {
                    // Heating was ON, now above target: turn OFF
                    self.last_relay_high = false;
                    0.0
                } else if !self.last_relay_high && current_value < self.config.target {
                    // Heating was OFF, now below target: turn ON
                    self.last_relay_high = true;
                    1.0
                } else {
                    // Maintain current state
                    if self.last_relay_high { 1.0 } else { 0.0 }
                };

                // Check if we have enough data to analyze
                if self.oscillation_data.len() > 40 {
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
        if self.oscillation_data.len() < 20 {
            return None;
        }

        // Find peaks and valleys using simple derivative method
        let mut peaks = Vec::new();
        let mut valleys = Vec::new();

        for i in 1..self.oscillation_data.len() - 1 {
            let prev = self.oscillation_data[i - 1].value;
            let curr = self.oscillation_data[i].value;
            let next = self.oscillation_data[i + 1].value;

            // Peak: higher than both neighbors
            if curr > prev && curr > next {
                peaks.push((self.oscillation_data[i].time, curr));
            }
            // Valley: lower than both neighbors
            else if curr < prev && curr < next {
                valleys.push((self.oscillation_data[i].time, curr));
            }
        }

        // Need at least the minimum number of oscillations
        let num_oscillations = peaks.len().min(valleys.len());
        if num_oscillations < self.config.min_oscillations {
            return None;
        }

        // Calculate ultimate period (average time between peaks)
        let mut periods = Vec::new();
        for i in 1..peaks.len() {
            periods.push(peaks[i].0 - peaks[i - 1].0);
        }

        if periods.is_empty() {
            return None;
        }

        let ultimate_period = periods.iter().sum::<f64>() / periods.len() as f64;

        // Calculate amplitude of oscillations
        let peak_avg = peaks.iter().map(|(_, v)| v).sum::<f64>() / peaks.len() as f64;
        let valley_avg = valleys.iter().map(|(_, v)| v).sum::<f64>() / valleys.len() as f64;
        let amplitude = (peak_avg - valley_avg) / 2.0;

        if amplitude < 0.1 {
            // Oscillations too small
            return None;
        }

        // Calculate ultimate gain using relay method formula
        // Ku = 4d / (π * a)
        // where d is relay amplitude (0 to 1 in our case = 1.0) and a is process oscillation amplitude
        let relay_amplitude = 1.0; // Full on/off relay
        let ultimate_gain = (4.0 * relay_amplitude) / (std::f64::consts::PI * amplitude);

        // Apply Ziegler-Nichols PID tuning rules
        // Classic Ziegler-Nichols (can be aggressive, but good starting point)
        let kp = 0.6 * ultimate_gain;
        let ki = 2.0 * kp / ultimate_period;
        let kd = kp * ultimate_period / 8.0;

        Some(AutoTuneResult {
            kp,
            ki,
            kd,
            ultimate_gain,
            ultimate_period,
        })
    }

    pub fn get_progress_percent(&self) -> f64 {
        match self.state {
            AutoTuneState::Idle => 0.0,
            AutoTuneState::MeasuringOscillations => {
                // Progress based on data collected (need ~100+ points for good analysis)
                (self.oscillation_data.len() as f64 / 150.0 * 100.0).min(99.0)
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
        assert_eq!(tuner.state, AutoTuneState::WaitingForSettle);
        assert!(tuner.is_active());
    }

    #[test]
    fn test_autotuner_relay_output() {
        let config = AutoTuneConfig {
            relay_amplitude: 0.4,
            settle_time_secs: 0.0,
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
        let (output, _) = tuner.update(200.0, later);
        assert!(output < 0.5);
    }
}
