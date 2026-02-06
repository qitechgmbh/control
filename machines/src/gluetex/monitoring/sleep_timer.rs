use super::config::SleepTimerConfig;
use crate::gluetex::OperationMode;
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
    /// Only active in Production mode - paused during Setup to allow setup
    /// Returns true if timer just triggered (state change)
    pub fn check(&mut self, operation_mode: OperationMode) -> bool {
        let now = Instant::now();

        if !self.config.enabled || operation_mode != OperationMode::Production {
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
    pub fn get_remaining_seconds(&self) -> u64 {
        if !self.config.enabled {
            return 0;
        }

        // If timer has been triggered, keep it at 0
        if self.triggered {
            return 0;
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
