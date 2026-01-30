use control_core::converters::angular_step_converter::AngularStepConverter;
use ethercat_hal::io::digital_input::DigitalInput;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use units::Length;
use units::angular_velocity::revolution_per_second;
use units::f64::AngularVelocity;
use units::length::millimeter;

/// State machine for homing and pattern control
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternControlState {
    /// Motor is idle or running in constant mode
    Idle,
    /// Motor is homing to the endstop
    Homing,
    /// Motor is running for Konturlänge distance
    Running,
    /// Motor has hit endstop and is paused
    Paused,
}

/// Controller for addon motors that follow the puller speed with a configurable ratio
#[derive(Debug)]
pub struct AddonMotorController {
    /// Whether the motor is enabled (Run mode)
    enabled: bool,
    /// Direction: true = forward, false = reverse
    forward: bool,
    /// Master ratio value (e.g., 2 in "2:1")
    master_ratio: f64,
    /// Slave ratio value (e.g., 1 in "2:1")
    slave_ratio: f64,
    /// Converter for angular velocity to steps
    converter: AngularStepConverter,
    /// Konturlänge in mm (0 = constant mode)
    konturlaenge_mm: f64,
    /// Pause in mm (0 = constant mode)
    pause_mm: f64,
    /// Current state in the pattern control state machine
    pattern_state: PatternControlState,
    /// Accumulated distance since last state change (mm)
    accumulated_distance: f64,
    /// Whether the motor needs homing on next enable
    needs_homing: bool,
}

impl AddonMotorController {
    /// Create a new addon motor controller
    ///
    /// # Arguments
    /// * `steps_per_revolution` - Number of steps per revolution for the stepper motor
    pub fn new(steps_per_revolution: i16) -> Self {
        Self {
            enabled: false,
            forward: true,
            master_ratio: 1.0,
            slave_ratio: 1.0,
            converter: AngularStepConverter::new(steps_per_revolution),
            konturlaenge_mm: 0.0,
            pause_mm: 0.0,
            pattern_state: PatternControlState::Idle,
            accumulated_distance: 0.0,
            needs_homing: false,
        }
    }

    /// Set whether the motor is enabled (Run mode)
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled && !self.enabled {
            // When enabling, check if we need homing
            if self.konturlaenge_mm > 0.0 || self.pause_mm > 0.0 {
                self.needs_homing = true;
                self.pattern_state = PatternControlState::Homing;
                self.accumulated_distance = 0.0;
            } else {
                self.pattern_state = PatternControlState::Idle;
            }
        } else if !enabled {
            // When disabling, reset state
            self.pattern_state = PatternControlState::Idle;
            self.accumulated_distance = 0.0;
        }
        self.enabled = enabled;
    }

    /// Get whether the motor is enabled
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set the rotation direction
    pub fn set_forward(&mut self, forward: bool) {
        self.forward = forward;
    }

    /// Get the rotation direction
    pub const fn is_forward(&self) -> bool {
        self.forward
    }

    /// Set the master ratio value
    ///
    /// The master ratio represents the puller rotations in the ratio.
    /// For example, in a 2:1 ratio, the master is 2.
    pub fn set_master_ratio(&mut self, master: f64) {
        self.master_ratio = master.max(0.1); // Prevent division issues
    }

    /// Set the slave ratio value
    ///
    /// The slave ratio represents the motor rotations in the ratio.
    /// For example, in a 2:1 ratio, the slave is 1.
    pub fn set_slave_ratio(&mut self, slave: f64) {
        self.slave_ratio = slave.max(0.1); // Prevent division issues
    }

    /// Get the master ratio value
    pub const fn get_master_ratio(&self) -> f64 {
        self.master_ratio
    }

    /// Get the slave ratio value
    pub const fn get_slave_ratio(&self) -> f64 {
        self.slave_ratio
    }

    /// Set Konturlänge in mm (0 = constant mode)
    pub fn set_konturlaenge_mm(&mut self, length_mm: f64) {
        self.konturlaenge_mm = length_mm.max(0.0);
        // Reset state when parameters change
        if self.enabled {
            if self.konturlaenge_mm > 0.0 || self.pause_mm > 0.0 {
                self.needs_homing = true;
                self.pattern_state = PatternControlState::Homing;
            } else {
                self.pattern_state = PatternControlState::Idle;
            }
            self.accumulated_distance = 0.0;
        }
    }

    /// Get Konturlänge in mm
    pub const fn get_konturlaenge_mm(&self) -> f64 {
        self.konturlaenge_mm
    }

    /// Set Pause in mm (0 = constant mode)
    pub fn set_pause_mm(&mut self, pause_mm: f64) {
        self.pause_mm = pause_mm.max(0.0);
        // Reset state when parameters change
        if self.enabled {
            if self.konturlaenge_mm > 0.0 || self.pause_mm > 0.0 {
                self.needs_homing = true;
                self.pattern_state = PatternControlState::Homing;
            } else {
                self.pattern_state = PatternControlState::Idle;
            }
            self.accumulated_distance = 0.0;
        }
    }

    /// Get Pause in mm
    pub const fn get_pause_mm(&self) -> f64 {
        self.pause_mm
    }

    /// Get current pattern control state
    pub const fn get_pattern_state(&self) -> PatternControlState {
        self.pattern_state
    }

    /// Calculate the motor angular velocity based on puller angular velocity and ratio
    ///
    /// # Arguments
    /// * `puller_angular_velocity` - The current angular velocity of the puller
    ///
    /// # Returns
    /// The angular velocity this motor should run at
    ///
    /// # Example
    /// If puller is rotating at 10 rev/s and ratio is 2:1 (master=2, slave=1),
    /// then motor should rotate at 5 rev/s (10 * 1/2)
    fn calculate_motor_velocity(
        &self,
        puller_angular_velocity: AngularVelocity,
    ) -> AngularVelocity {
        if !self.enabled {
            return AngularVelocity::new::<revolution_per_second>(0.0);
        }

        // Motor velocity = puller velocity * (slave_ratio / master_ratio)
        // For 2:1, motor rotates at half the puller speed (1/2)
        // For 1:2, motor rotates at twice the puller speed (2/1)
        let ratio = self.slave_ratio / self.master_ratio;
        let velocity = puller_angular_velocity * ratio;

        // Apply direction: negative velocity for reverse
        if self.forward { velocity } else { -velocity }
    }

    /// Update the motor speed based on the puller angular velocity and optionally handle homing/pattern control
    ///
    /// # Arguments
    /// * `motor` - The stepper motor to control
    /// * `puller_angular_velocity` - The current angular velocity of the puller
    /// * `endstop` - Optional endstop for homing and pattern control
    /// * `puller_length_moved` - Length moved by the puller (for distance tracking in pattern mode)
    pub fn sync_motor_speed(
        &mut self,
        motor: &mut StepperVelocityEL70x1,
        puller_angular_velocity: AngularVelocity,
        endstop: Option<&DigitalInput>,
        puller_length_moved: Length,
    ) {
        if !self.enabled {
            // If disabled, ensure motor is not enabled
            if motor.is_enabled() {
                motor.set_enabled(false);
            }
            return;
        }

        // Enable motor if not already enabled
        if !motor.is_enabled() {
            motor.set_enabled(true);
        }

        // Check if we're in pattern control mode
        let pattern_mode = self.konturlaenge_mm > 0.0 || self.pause_mm > 0.0;

        if pattern_mode && endstop.is_some() {
            self.handle_pattern_control(
                motor,
                puller_angular_velocity,
                endstop.unwrap(),
                puller_length_moved,
            );
        } else {
            // Constant mode - just follow the ratio
            let target_velocity = self.calculate_motor_velocity(puller_angular_velocity);
            let steps_per_second = self.converter.angular_velocity_to_steps(target_velocity);
            let _ = motor.set_speed(steps_per_second);
        }
    }

    /// Handle pattern control state machine
    fn handle_pattern_control(
        &mut self,
        motor: &mut StepperVelocityEL70x1,
        puller_angular_velocity: AngularVelocity,
        endstop: &DigitalInput,
        puller_length_moved: Length,
    ) {
        // Update accumulated distance
        let distance_mm = puller_length_moved.get::<millimeter>();

        match self.pattern_state {
            PatternControlState::Homing => {
                // Check if endstop is hit
                if endstop.get_value().unwrap_or(false) {
                    // Endstop hit - stop and transition to Running
                    let _ = motor.set_speed(0.0);
                    self.pattern_state = PatternControlState::Running;
                    self.accumulated_distance = 0.0;
                } else {
                    // Keep moving towards endstop
                    let target_velocity = self.calculate_motor_velocity(puller_angular_velocity);
                    let steps_per_second =
                        self.converter.angular_velocity_to_steps(target_velocity);
                    let _ = motor.set_speed(steps_per_second);
                }
            }
            PatternControlState::Running => {
                self.accumulated_distance += distance_mm;

                if self.accumulated_distance >= self.konturlaenge_mm {
                    // Konturlänge reached - go back to endstop
                    let target_velocity = self.calculate_motor_velocity(puller_angular_velocity);
                    let steps_per_second =
                        self.converter.angular_velocity_to_steps(target_velocity);
                    let _ = motor.set_speed(steps_per_second);

                    // Check if we've reached the endstop
                    if endstop.get_value().unwrap_or(false) {
                        let _ = motor.set_speed(0.0);
                        self.pattern_state = PatternControlState::Paused;
                        self.accumulated_distance = 0.0;
                    }
                } else {
                    // Continue running
                    let target_velocity = self.calculate_motor_velocity(puller_angular_velocity);
                    let steps_per_second =
                        self.converter.angular_velocity_to_steps(target_velocity);
                    let _ = motor.set_speed(steps_per_second);
                }
            }
            PatternControlState::Paused => {
                // Motor is stopped at endstop
                let _ = motor.set_speed(0.0);
                self.accumulated_distance += distance_mm;

                if self.accumulated_distance >= self.pause_mm {
                    // Pause duration reached - start running again
                    self.pattern_state = PatternControlState::Running;
                    self.accumulated_distance = 0.0;
                }
            }
            PatternControlState::Idle => {
                // Should not reach here in pattern mode, but handle it anyway
                let target_velocity = self.calculate_motor_velocity(puller_angular_velocity);
                let steps_per_second = self.converter.angular_velocity_to_steps(target_velocity);
                let _ = motor.set_speed(steps_per_second);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use units::angular_velocity::revolution_per_second;

    #[test]
    fn test_addon_motor_controller_disabled() {
        let controller = AddonMotorController::new(200);
        let puller_velocity = AngularVelocity::new::<revolution_per_second>(10.0);

        let motor_velocity = controller.calculate_motor_velocity(puller_velocity);

        assert_eq!(
            motor_velocity.get::<revolution_per_second>(),
            0.0,
            "Motor should be stopped when disabled"
        );
    }

    #[test]
    fn test_addon_motor_controller_ratio_2_to_1() {
        let mut controller = AddonMotorController::new(200);
        controller.set_enabled(true);
        controller.set_master_ratio(2.0);
        controller.set_slave_ratio(1.0);

        let puller_velocity = AngularVelocity::new::<revolution_per_second>(10.0);
        let motor_velocity = controller.calculate_motor_velocity(puller_velocity);

        // For every 2 puller rotations, motor should rotate 1 time
        // So motor speed should be half of puller speed
        assert_eq!(
            motor_velocity.get::<revolution_per_second>(),
            5.0,
            "Motor should run at half speed with 2:1 ratio"
        );
    }

    #[test]
    fn test_addon_motor_controller_ratio_1_to_2() {
        let mut controller = AddonMotorController::new(200);
        controller.set_enabled(true);
        controller.set_master_ratio(1.0);
        controller.set_slave_ratio(2.0);

        let puller_velocity = AngularVelocity::new::<revolution_per_second>(10.0);
        let motor_velocity = controller.calculate_motor_velocity(puller_velocity);

        // For every 1 puller rotation, motor should rotate 2 times
        // So motor speed should be twice the puller speed
        assert_eq!(
            motor_velocity.get::<revolution_per_second>(),
            20.0,
            "Motor should run at twice speed with 1:2 ratio"
        );
    }

    #[test]
    fn test_addon_motor_controller_ratio_3_to_2() {
        let mut controller = AddonMotorController::new(200);
        controller.set_enabled(true);
        controller.set_master_ratio(3.0);
        controller.set_slave_ratio(2.0);

        let puller_velocity = AngularVelocity::new::<revolution_per_second>(9.0);
        let motor_velocity = controller.calculate_motor_velocity(puller_velocity);

        // For every 3 puller rotations, motor should rotate 2 times
        // So motor speed should be 2/3 of puller speed
        // 9 * (2/3) = 6.0
        assert_eq!(
            motor_velocity.get::<revolution_per_second>(),
            6.0,
            "Motor should run at 2/3 speed with 3:2 ratio"
        );
    }

    #[test]
    fn test_addon_motor_controller_ratio_1_to_1() {
        let mut controller = AddonMotorController::new(200);
        controller.set_enabled(true);
        controller.set_master_ratio(1.0);
        controller.set_slave_ratio(1.0);

        let puller_velocity = AngularVelocity::new::<revolution_per_second>(10.0);
        let motor_velocity = controller.calculate_motor_velocity(puller_velocity);

        // 1:1 ratio means motor runs at same speed as puller
        assert_eq!(
            motor_velocity.get::<revolution_per_second>(),
            10.0,
            "Motor should run at same speed with 1:1 ratio"
        );
    }
}
