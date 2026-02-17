use super::config::VoltageMonitorConfig;
use crate::gluetex::OperationMode;
use std::time::Instant;

/// A circular buffer entry containing a voltage reading and the distance at which it was recorded
#[derive(Debug, Clone, Copy)]
struct VoltageHistoryEntry {
    voltage: f64,
    distance_mm: f64,
}

/// Voltage monitoring state with support for delayed readings
#[derive(Debug)]
pub struct VoltageMonitor {
    pub config: VoltageMonitorConfig,
    pub triggered: bool,
    /// Time when voltage first went out of range (used for debouncing)
    pub out_of_range_since: Option<Instant>,
    name: String,
    /// Circular buffer storing historical voltage readings (max 1000 entries)
    voltage_history: Vec<VoltageHistoryEntry>,
    /// Current position in the circular buffer
    history_index: usize,
    /// Total distance accumulated since creation
    accumulated_distance_mm: f64,
}

impl VoltageMonitor {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: VoltageMonitorConfig::default(),
            triggered: false,
            out_of_range_since: None,
            name: name.into(),
            voltage_history: vec![
                VoltageHistoryEntry {
                    voltage: 0.0,
                    distance_mm: 0.0
                };
                1000
            ],
            history_index: 0,
            accumulated_distance_mm: 0.0,
        }
    }

    /// Record a voltage reading at the current distance
    /// Should be called once per control loop iteration
    pub fn record_voltage(&mut self, voltage: f64, distance_traveled_mm: f64) {
        self.accumulated_distance_mm += distance_traveled_mm;

        self.voltage_history[self.history_index] = VoltageHistoryEntry {
            voltage,
            distance_mm: self.accumulated_distance_mm,
        };

        self.history_index = (self.history_index + 1) % self.voltage_history.len();
    }

    /// Get the voltage reading from delay_mm ago
    /// If delay is larger than the recorded history, returns the oldest available reading
    pub fn get_delayed_voltage(&self) -> f64 {
        if self.accumulated_distance_mm < self.config.delay_mm {
            // Not enough history yet, return the oldest reading we have
            return self.voltage_history[(self.history_index + 1) % self.voltage_history.len()]
                .voltage;
        }

        let target_distance = self.accumulated_distance_mm - self.config.delay_mm;
        let mut closest_entry = self.voltage_history[self.history_index];
        let mut closest_diff = f64::MAX;

        // Find the entry closest to the target distance
        for entry in &self.voltage_history {
            let diff = (entry.distance_mm - target_distance).abs();
            if diff < closest_diff {
                closest_diff = diff;
                closest_entry = *entry;
            }
        }

        closest_entry.voltage
    }

    /// Check voltage and trigger emergency stop if limits exceeded
    /// Uses a 200ms debounce to prevent false triggers from momentary spikes
    /// Returns (new_triggered_state, state_changed)
    pub fn check(&mut self, _voltage: f64, operation_mode: OperationMode) -> (bool, bool) {
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

        // Use delayed voltage for checking limits
        let check_voltage = self.get_delayed_voltage();

        // Check if this voltage is out of range
        let is_out_of_range = check_voltage < min_voltage || check_voltage > max_voltage;

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
                        "{} voltage monitor triggered after 200ms! Delayed voltage: {:.2}V (limits: {:.2}V-{:.2}V)",
                        self.name,
                        check_voltage,
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
