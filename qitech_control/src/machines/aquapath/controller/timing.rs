use std::time::Duration;
use std::time::Instant;

/// A restartable one-shot timer used for the controller's grace periods and holds.
///
/// The duration is supplied per call rather than stored, because several of them are
/// retuned at runtime through the machine config and must not be mirrored here.
#[derive(Debug, Clone, Copy, Default)]
pub struct Timer {
    started_at: Option<Instant>,
}

impl Timer {
    /// Starts the timer unless it is already running, and reports the start instant.
    pub fn start_if_idle(&mut self, now: Instant) -> Instant {
        *self.started_at.get_or_insert(now)
    }

    pub fn restart(&mut self, now: Instant) {
        self.started_at = Some(now);
    }

    pub fn clear(&mut self) {
        self.started_at = None;
    }

    pub fn is_running(&self) -> bool {
        self.started_at.is_some()
    }

    /// An idle timer reports `false`: something that never started has never finished.
    pub fn has_elapsed(&self, now: Instant, duration: Duration) -> bool {
        self.started_at
            .is_some_and(|started_at| now.duration_since(started_at) >= duration)
    }

    /// `None` while idle, so each caller decides what an unstarted timer should report.
    pub fn remaining(&self, now: Instant, duration: Duration) -> Option<Duration> {
        self.started_at
            .map(|started_at| duration.saturating_sub(now.duration_since(started_at)))
    }

    /// Whether the timer started within the last `duration` — "did this happen recently".
    pub fn started_within(&self, now: Instant, duration: Duration) -> bool {
        self.started_at
            .is_some_and(|started_at| now.duration_since(started_at) < duration)
    }
}
