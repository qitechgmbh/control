use super::config::TensionArmMonitorConfig;
use crate::gluetex::OperationMode;
use std::time::Instant;
use units::Angle;
use units::angle::degree;

/// Tension arm monitoring state
#[derive(Debug)]
pub struct TensionArmMonitor {
    pub config: TensionArmMonitorConfig,
    pub triggered: bool,
    /// Time when tension arm first went out of range (used for debouncing)
    pub out_of_range_since: Option<Instant>,
    name: String,
}

impl TensionArmMonitor {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: TensionArmMonitorConfig::default(),
            triggered: false,
            out_of_range_since: None,
            name: name.into(),
        }
    }

    /// Check tension arm position and trigger emergency stop if out of range
    /// Uses a 200ms debounce to prevent false triggers from momentary spikes
    /// Returns (new_triggered_state, state_changed)
    pub fn check(&mut self, angle: Angle, operation_mode: OperationMode) -> (bool, bool) {
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

        let min_angle = self.config.min_angle;
        let max_angle = self.config.max_angle;

        // Check if this tension arm is out of range
        let is_out_of_range = angle < min_angle || angle > max_angle;

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
                        "{} tension arm monitor triggered after 200ms! Angle: {:.1}° (limits: {:.1}°-{:.1}°)",
                        self.name,
                        angle.get::<degree>(),
                        min_angle.get::<degree>(),
                        max_angle.get::<degree>()
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
                    "{} tension arm monitor cleared - arm back in range",
                    self.name
                );
                self.triggered = false;
                return (false, true); // state changed
            }

            return (false, false); // no change
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn triggers_after_debounce_when_out_of_limits_in_production() {
        let mut monitor = TensionArmMonitor::new("test");
        monitor.config.enabled = true;
        monitor.config.min_angle = Angle::new::<degree>(10.0);
        monitor.config.max_angle = Angle::new::<degree>(170.0);

        // First out-of-range check starts debounce timer.
        let (triggered, changed) =
            monitor.check(Angle::new::<degree>(180.0), OperationMode::Production);
        assert!(!triggered);
        assert!(!changed);
        assert!(monitor.out_of_range_since.is_some());

        // Simulate elapsed debounce period deterministically.
        monitor.out_of_range_since = Some(Instant::now() - Duration::from_millis(250));
        let (triggered, changed) =
            monitor.check(Angle::new::<degree>(180.0), OperationMode::Production);
        assert!(triggered);
        assert!(changed);
        assert!(monitor.triggered);
    }

    #[test]
    fn clears_trigger_when_back_in_range() {
        let mut monitor = TensionArmMonitor::new("test");
        monitor.config.enabled = true;
        monitor.triggered = true;
        monitor.out_of_range_since = Some(Instant::now());

        let (triggered, changed) =
            monitor.check(Angle::new::<degree>(90.0), OperationMode::Production);
        assert!(!triggered);
        assert!(changed);
        assert!(!monitor.triggered);
        assert!(monitor.out_of_range_since.is_none());
    }

    #[test]
    fn disabled_or_setup_mode_prevents_triggering() {
        let mut monitor = TensionArmMonitor::new("test");
        monitor.config.enabled = false;

        let (triggered, changed) =
            monitor.check(Angle::new::<degree>(180.0), OperationMode::Production);
        assert!(!triggered);
        assert!(!changed);

        monitor.config.enabled = true;
        let (triggered, changed) = monitor.check(Angle::new::<degree>(180.0), OperationMode::Setup);
        assert!(!triggered);
        assert!(!changed);
    }
}
