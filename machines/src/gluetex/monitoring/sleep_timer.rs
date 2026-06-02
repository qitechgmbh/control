use super::super::OperationMode;
use super::config::SleepTimerConfig;
use std::time::Instant;

/// Sleep timer monitoring state
#[derive(Debug)]
pub struct SleepTimer {
    pub config: SleepTimerConfig,
    pub triggered: bool,
    pub last_activity_time: Instant,
}

impl Default for SleepTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl SleepTimer {
    pub fn new() -> Self {
        Self {
            config: SleepTimerConfig::default(),
            triggered: false,
            last_activity_time: Instant::now(),
        }
    }

    /// Check if sleep timer has expired
    /// Only counts down in Setup mode
    /// Returns true if timer just triggered (state change)
    pub fn check(&mut self, operation_mode: OperationMode) -> bool {
        let now = Instant::now();

        if !self.config.enabled {
            self.triggered = false;
            return false;
        }

        // Only count down in Setup mode
        // Reset timer when not in Setup mode
        if operation_mode != OperationMode::Setup {
            self.last_activity_time = now;
            self.triggered = false;
            return false;
        }

        let elapsed = now.duration_since(self.last_activity_time).as_secs();

        if elapsed >= self.config.timeout_seconds && !self.triggered {
            tracing::info!("Sleep timer expired - entering standby mode");
            self.triggered = true;
            return true; // State changed
        }

        false
    }

    /// Get remaining seconds on sleep timer
    /// Only counts down in Setup mode, returns full time otherwise
    pub fn get_remaining_seconds(&self, operation_mode: OperationMode) -> u64 {
        if !self.config.enabled {
            return 0;
        }

        // If timer has been triggered, keep it at 0
        if self.triggered {
            return 0;
        }

        // Return full time when not in Setup mode
        if operation_mode != OperationMode::Setup {
            return self.config.timeout_seconds;
        }

        let elapsed = Instant::now()
            .duration_since(self.last_activity_time)
            .as_secs();

        if elapsed >= self.config.timeout_seconds {
            0
        } else {
            self.config.timeout_seconds - elapsed
        }
    }

    /// Reset the sleep timer (mark activity)
    pub fn reset(&mut self) {
        self.last_activity_time = Instant::now();
        self.triggered = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gluetex::OperationMode;
    use std::time::Duration;

    #[test]
    fn check_triggers_when_timeout_expires_in_setup_mode() {
        let mut timer = SleepTimer::new();
        timer.config.enabled = true;
        timer.config.timeout_seconds = 5;
        timer.last_activity_time = Instant::now() - Duration::from_secs(6);

        let changed = timer.check(OperationMode::Setup);
        assert!(changed, "timer should report state change when it expires");
        assert!(timer.triggered, "timer should be marked as triggered");
        assert_eq!(timer.get_remaining_seconds(OperationMode::Setup), 0);
    }

    #[test]
    fn check_does_not_trigger_outside_setup_and_resets() {
        let mut timer = SleepTimer::new();
        timer.config.enabled = true;
        timer.config.timeout_seconds = 5;
        timer.triggered = true;
        timer.last_activity_time = Instant::now() - Duration::from_secs(10);

        let changed = timer.check(OperationMode::Production);
        assert!(!changed, "no trigger transition should be reported");
        assert!(
            !timer.triggered,
            "triggered flag should be cleared outside setup mode"
        );
        assert_eq!(timer.get_remaining_seconds(OperationMode::Production), 5);
    }

    #[test]
    fn disabled_timer_never_triggers_and_reports_zero_remaining() {
        let mut timer = SleepTimer::new();
        timer.config.enabled = false;
        timer.config.timeout_seconds = 5;
        timer.last_activity_time = Instant::now() - Duration::from_secs(10);

        let changed = timer.check(OperationMode::Setup);
        assert!(!changed);
        assert!(!timer.triggered);
        assert_eq!(timer.get_remaining_seconds(OperationMode::Setup), 0);
    }
}
