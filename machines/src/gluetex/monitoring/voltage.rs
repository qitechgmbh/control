use super::config::VoltageMonitorConfig;
use crate::gluetex::OperationMode;
use std::time::Instant;

/// Voltage monitoring state
#[derive(Debug)]
pub struct VoltageMonitor {
    pub config: VoltageMonitorConfig,
    pub triggered: bool,
    /// Time when voltage first went out of range (used for debouncing)
    pub out_of_range_since: Option<Instant>,
    name: String,
}

impl VoltageMonitor {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: VoltageMonitorConfig::default(),
            triggered: false,
            out_of_range_since: None,
            name: name.into(),
        }
    }

    /// Check voltage and trigger emergency stop if limits exceeded
    /// Uses a 200ms debounce to prevent false triggers from momentary spikes
    /// Returns (new_triggered_state, state_changed)
    pub fn check(
        &mut self,
        voltage: f64,
        operation_mode: OperationMode,
    ) -> (bool, bool) {
        let now = Instant::now();
        let in_production_mode = operation_mode == OperationMode::Production;

        // Only check if monitoring is enabled AND in Production mode
        if !self.config.enabled || !in_production_mode {
            // Clear debounce timer
            self.out_of_range_since = None;

            // Clear triggered flag if monitoring is disabled or not in Production mode
            if self.triggered {
                self.triggered = false;
                return (false, true); // state changed
            }
            return (false, false); // no change
        }

        let min_voltage = self.config.min_voltage;
        let max_voltage = self.config.max_voltage;

        // Check if this voltage is out of range
        let is_out_of_range = voltage < min_voltage || voltage > max_voltage;

        if is_out_of_range {
            // Start or continue tracking out-of-range time
            if self.out_of_range_since.is_none() {
                self.out_of_range_since = Some(now);
            }

            // Check if out of range for more than 200ms
            if let Some(start_time) = self.out_of_range_since {
                let duration = now.duration_since(start_time);
                if duration.as_millis() >= 200 && !self.triggered {
                    // Trigger emergency stop after debounce period
                    tracing::warn!(
                        "{} voltage monitor triggered after 200ms! Voltage: {:.2}V (limits: {:.2}V-{:.2}V)",
                        self.name,
                        voltage,
                        min_voltage,
                        max_voltage
                    );
                    self.triggered = true;
                    return (true, true); // triggered and state changed
                }
            }
            return (self.triggered, false); // no change yet (still within debounce)
        } else {
            // Back in range - clear debounce timer
            if self.out_of_range_since.is_some() {
                self.out_of_range_since = None;
            }

            // Clear triggered flag if back in range
            if self.triggered {
                tracing::info!(
                    "{} voltage monitor cleared - voltage back in range",
                    self.name
                );
                self.triggered = false;
                return (false, true); // state changed
            }

            return (false, false); // no change
        }
    }
}
