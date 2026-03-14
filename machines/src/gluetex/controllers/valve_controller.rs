use ethercat_hal::io::digital_output::DigitalOutput;
use units::Length;
use units::length::millimeter;

/// Controller for a valve that turns on for X mm and off for Y mm based on puller movement
#[derive(Debug)]
pub struct ValveController {
    /// Whether the valve control is enabled
    enabled: bool,
    /// Manual override: when Some, this value overrides pattern-based control
    manual_override: Option<bool>,
    /// Distance in mm that the valve should be ON (0 = disabled pattern)
    on_distance_mm: f64,
    /// Distance in mm that the valve should be OFF (0 = disabled pattern)
    off_distance_mm: f64,
    /// Current state of the pattern: true = in ON phase, false = in OFF phase
    pattern_state: bool,
    /// Accumulated distance in current phase (mm)
    accumulated_distance: f64,
}

impl ValveController {
    /// Create a new valve controller
    pub fn new() -> Self {
        Self {
            enabled: false,
            manual_override: None,
            on_distance_mm: 0.0,
            off_distance_mm: 0.0,
            pattern_state: false,
            accumulated_distance: 0.0,
        }
    }

    /// Set whether the valve control is enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            // When disabling, reset state
            self.accumulated_distance = 0.0;
            self.pattern_state = false;
        }
        self.enabled = enabled;
    }

    /// Get whether the valve control is enabled
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set manual override value
    /// - Some(true) = manually force valve ON
    /// - Some(false) = manually force valve OFF
    /// - None = use automatic pattern control
    pub fn set_manual_override(&mut self, manual: Option<bool>) {
        self.manual_override = manual;
    }

    /// Get the manual override value
    pub const fn get_manual_override(&self) -> Option<bool> {
        self.manual_override
    }

    /// Set the ON distance in mm
    pub fn set_on_distance_mm(&mut self, distance_mm: f64) {
        self.on_distance_mm = distance_mm.max(0.0);
        // Reset state when parameters change
        if self.enabled && self.manual_override.is_none() {
            self.accumulated_distance = 0.0;
            self.pattern_state = false;
        }
    }

    /// Get the ON distance in mm
    pub const fn get_on_distance_mm(&self) -> f64 {
        self.on_distance_mm
    }

    /// Set the OFF distance in mm
    pub fn set_off_distance_mm(&mut self, distance_mm: f64) {
        self.off_distance_mm = distance_mm.max(0.0);
        // Reset state when parameters change
        if self.enabled && self.manual_override.is_none() {
            self.accumulated_distance = 0.0;
            self.pattern_state = false;
        }
    }

    /// Get the OFF distance in mm
    pub const fn get_off_distance_mm(&self) -> f64 {
        self.off_distance_mm
    }

    /// Get the current pattern state
    pub const fn get_pattern_state(&self) -> bool {
        self.pattern_state
    }

    /// Get the accumulated distance in the current phase
    pub const fn get_accumulated_distance(&self) -> f64 {
        self.accumulated_distance
    }

    /// Update the valve state based on puller movement
    ///
    /// # Arguments
    /// * `valve` - The digital output controlling the valve
    /// * `puller_length_moved` - Length moved by the puller since last call (for distance tracking)
    pub fn update_valve(&mut self, valve: &mut DigitalOutput, puller_length_moved: Length) {
        // Check if manual override is active
        if let Some(manual_state) = self.manual_override {
            valve.set(manual_state);
            return;
        }

        // If not enabled or no pattern configured, turn valve off
        if !self.enabled || (self.on_distance_mm == 0.0 && self.off_distance_mm == 0.0) {
            valve.set(false);
            return;
        }

        // Update accumulated distance
        let distance_mm = puller_length_moved.get::<millimeter>();
        self.accumulated_distance += distance_mm;

        // Check if we need to switch phases
        let target_distance = if self.pattern_state {
            self.on_distance_mm
        } else {
            self.off_distance_mm
        };

        if self.accumulated_distance >= target_distance {
            // Switch to the other phase
            self.pattern_state = !self.pattern_state;
            self.accumulated_distance = 0.0;
        }

        // Set valve based on current pattern state
        valve.set(self.pattern_state);
    }

    /// Get the current valve state (what it should be set to)
    /// This is useful for monitoring/display purposes
    pub fn get_desired_state(&self) -> bool {
        if let Some(manual_state) = self.manual_override {
            return manual_state;
        }

        if !self.enabled || (self.on_distance_mm == 0.0 && self.off_distance_mm == 0.0) {
            return false;
        }

        self.pattern_state
    }
}

impl Default for ValveController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethercat_hal::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};
    use smol::lock::RwLock;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    #[derive(Clone, Copy)]
    struct TestPort;

    struct TestDigitalOutputDevice {
        state: Arc<AtomicBool>,
    }

    impl DigitalOutputDevice<TestPort> for TestDigitalOutputDevice {
        fn set_output(&mut self, _port: TestPort, value: DigitalOutputOutput) {
            self.state.store(value.0, Ordering::SeqCst);
        }

        fn get_output(&self, _port: TestPort) -> DigitalOutputOutput {
            DigitalOutputOutput(self.state.load(Ordering::SeqCst))
        }
    }

    fn create_digital_output() -> (DigitalOutput, Arc<AtomicBool>) {
        let shared_state = Arc::new(AtomicBool::new(false));
        let device: Arc<RwLock<dyn DigitalOutputDevice<TestPort>>> =
            Arc::new(RwLock::new(TestDigitalOutputDevice {
                state: shared_state.clone(),
            }));
        (DigitalOutput::new(device, TestPort), shared_state)
    }

    #[test]
    fn manual_override_has_priority() {
        let mut controller = ValveController::new();
        controller.set_enabled(false);
        controller.set_manual_override(Some(true));

        let (mut valve, shared_state) = create_digital_output();
        controller.update_valve(&mut valve, Length::new::<millimeter>(100.0));

        assert!(shared_state.load(Ordering::SeqCst));
        assert!(controller.get_desired_state());
    }

    #[test]
    fn pattern_switches_between_off_and_on_phases_by_distance() {
        let mut controller = ValveController::new();
        controller.set_enabled(true);
        controller.set_on_distance_mm(2.0);
        controller.set_off_distance_mm(3.0);

        let (mut valve, shared_state) = create_digital_output();

        // OFF phase, below threshold
        controller.update_valve(&mut valve, Length::new::<millimeter>(1.0));
        assert!(!shared_state.load(Ordering::SeqCst));

        // OFF phase reaches threshold -> switch to ON
        controller.update_valve(&mut valve, Length::new::<millimeter>(2.0));
        assert!(shared_state.load(Ordering::SeqCst));

        // ON phase reaches threshold -> switch to OFF
        controller.update_valve(&mut valve, Length::new::<millimeter>(2.0));
        assert!(!shared_state.load(Ordering::SeqCst));
    }

    #[test]
    fn disabling_resets_pattern_state() {
        let mut controller = ValveController::new();
        controller.set_enabled(true);
        controller.set_on_distance_mm(1.0);
        controller.set_off_distance_mm(1.0);

        let (mut valve, shared_state) = create_digital_output();
        controller.update_valve(&mut valve, Length::new::<millimeter>(1.0));
        assert!(shared_state.load(Ordering::SeqCst));

        controller.set_enabled(false);
        controller.update_valve(&mut valve, Length::new::<millimeter>(1.0));
        assert!(!shared_state.load(Ordering::SeqCst));
        assert!(!controller.get_pattern_state());
        assert_eq!(controller.get_accumulated_distance(), 0.0);
    }
}
