use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ValveController {
    /// Current state of the valve (on/off)
    active: bool,
    /// When the valve should be turned off (if Some, valve is in pulse mode)
    turn_off_time: Option<Instant>,
}

impl ValveController {
    pub fn new() -> Self {
        Self {
            active: false,
            turn_off_time: None,
        }
    }

    /// Get current valve state
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Set valve to permanently on state
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        self.turn_off_time = None;
    }

    /// Activate valve for a specific duration (pulse mode)
    pub fn activate_pulse(&mut self, duration_ms: u64) {
        self.active = true;
        self.turn_off_time = Some(Instant::now() + Duration::from_millis(duration_ms));
    }

    /// Update valve state based on current time
    /// Returns true if the valve state changed
    pub fn update(&mut self, now: Instant) -> bool {
        if let Some(turn_off_time) = self.turn_off_time {
            if now >= turn_off_time {
                // Time to turn off the valve
                self.active = false;
                self.turn_off_time = None;
                return true; // State changed
            }
        }
        false // No state change
    }

    /// Check if valve is in pulse mode (has a timer set)
    pub fn is_in_pulse_mode(&self) -> bool {
        self.turn_off_time.is_some()
    }

    /// Get remaining pulse time in milliseconds (if in pulse mode)
    pub fn remaining_pulse_time_ms(&self, now: Instant) -> Option<u64> {
        if let Some(turn_off_time) = self.turn_off_time {
            if now < turn_off_time {
                return Some(turn_off_time.duration_since(now).as_millis() as u64);
            }
        }
        None
    }
}

impl Default for ValveController {
    fn default() -> Self {
        Self::new()
    }
}
