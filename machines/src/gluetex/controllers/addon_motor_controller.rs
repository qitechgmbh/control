use control_core::converters::angular_step_converter::AngularStepConverter;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use units::Length;
use units::angular_velocity::revolution_per_minute;
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
    /// Whether the motor is in manual homing mode (triggered by user button)
    manual_homing: bool,
    /// Whether the sensor has been seen as false after kontur length was reached
    /// (prevents stopping immediately if sensor is already true when kontur length is reached)
    seen_sensor_clear_after_kontur: bool,
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
            manual_homing: false,
            seen_sensor_clear_after_kontur: false,
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

    /// Start manual homing - moves motor in positive direction until endstop is hit
    ///
    /// This method initiates a manual homing sequence, putting the motor into
    /// the Homing state. The motor will move in the positive direction at a fixed
    /// speed until the endstop is triggered, then automatically go to standby.
    pub fn start_manual_homing(&mut self) {
        self.enabled = true;
        self.manual_homing = true;
        self.pattern_state = PatternControlState::Homing;
        self.accumulated_distance = 0.0;
        self.needs_homing = false;
    }

    /// Reset runtime state after a safety stop while preserving user configuration.
    ///
    /// If the controller is still user-enabled and pattern mode is active, this re-arms
    /// homing so restart transitions cannot resume from stale `Paused`/`Running` state.
    pub fn on_safety_stop(&mut self) {
        self.manual_homing = false;
        self.accumulated_distance = 0.0;
        self.seen_sensor_clear_after_kontur = false;

        let pattern_mode = self.konturlaenge_mm > 0.0 || self.pause_mm > 0.0;
        if self.enabled && pattern_mode {
            self.needs_homing = true;
            self.pattern_state = PatternControlState::Homing;
        } else {
            self.needs_homing = false;
            self.pattern_state = PatternControlState::Idle;
        }
    }

    /// Convert raw stepper steps/s to RPM for this motor
    pub fn steps_to_rpm(&self, steps: i32) -> f64 {
        self.converter
            .steps_to_angular_velocity(steps as f64)
            .get::<revolution_per_minute>()
            .abs()
    }

    /// Convert raw stepper steps/s to reference RPM without addon ratio scaling.
    ///
    /// This is used for UI display when RPM should reflect the base puller-referenced
    /// speed instead of the ratio-scaled motor speed.
    pub fn steps_to_reference_rpm(&self, steps: i32) -> f64 {
        let motor_rpm = self.steps_to_rpm(steps);
        let ratio = (self.slave_ratio / self.master_ratio).abs();

        if ratio > f64::EPSILON {
            motor_rpm / ratio
        } else {
            motor_rpm
        }
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
    /// * `endstop_hit` - Optional boolean indicating if endstop is triggered (for homing and pattern control)
    /// * `puller_length_moved` - Length moved by the puller (for distance tracking in pattern mode)
    pub fn sync_motor_speed(
        &mut self,
        motor: &mut StepperVelocityEL70x1,
        puller_angular_velocity: AngularVelocity,
        endstop_hit: Option<bool>,
        puller_length_moved: Length,
    ) {
        // Handle manual homing first (highest priority)
        if self.manual_homing {
            // Enable motor if not already enabled
            if !motor.is_enabled() {
                motor.set_enabled(true);
            }

            if let Some(endstop_triggered) = endstop_hit {
                if endstop_triggered {
                    // Endstop hit - stop motor, disable, and exit manual homing
                    let _ = motor.set_speed(0.0);
                    motor.set_enabled(false);
                    self.enabled = false;
                    self.manual_homing = false;
                    self.pattern_state = PatternControlState::Idle;
                    return;
                }
            }

            // Move towards endstop at fixed speed (0.5 rev/s regardless of puller speed)
            let homing_velocity = AngularVelocity::new::<revolution_per_second>(0.5);
            let steps_per_second = self.converter.angular_velocity_to_steps(homing_velocity);
            let _ = motor.set_speed(steps_per_second);
            return;
        }

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

        if pattern_mode && endstop_hit.is_some() {
            self.handle_pattern_control(
                motor,
                puller_angular_velocity,
                endstop_hit.unwrap(),
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
        endstop_hit: bool,
        puller_length_moved: Length,
    ) {
        // Update accumulated distance
        let distance_mm = puller_length_moved.get::<millimeter>();

        match self.pattern_state {
            PatternControlState::Homing => {
                // Check if endstop is hit
                if endstop_hit {
                    // Endstop hit - stop and transition to Running
                    let _ = motor.set_speed(0.0);
                    self.pattern_state = PatternControlState::Running;
                    self.accumulated_distance = 0.0;
                    self.seen_sensor_clear_after_kontur = false;
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
                    // Konturlänge reached - keep running until we see sensor go false,
                    // then stop on the next true reading
                    let target_velocity = self.calculate_motor_velocity(puller_angular_velocity);
                    let steps_per_second =
                        self.converter.angular_velocity_to_steps(target_velocity);
                    let _ = motor.set_speed(steps_per_second);

                    if !endstop_hit {
                        // Sensor cleared - now we can accept the next true as a valid stop
                        self.seen_sensor_clear_after_kontur = true;
                    } else if self.seen_sensor_clear_after_kontur {
                        // Sensor is true AND we already saw it go false - valid stop
                        let _ = motor.set_speed(0.0);
                        self.pattern_state = PatternControlState::Paused;
                        self.accumulated_distance = 0.0;
                        self.seen_sensor_clear_after_kontur = false;
                    }
                    // else: sensor is true but we haven't seen it go false yet - keep running
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
                    self.seen_sensor_clear_after_kontur = false;
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
    use ethercat_hal::io::stepper_velocity_el70x1::{
        StepperVelocityEL70x1Device, StepperVelocityEL70x1Input, StepperVelocityEL70x1Output,
    };
    use ethercat_hal::shared_config::el70x1::EL70x1SpeedRange;
    use smol::lock::RwLock;
    use std::sync::Arc;
    use units::angular_velocity::revolution_per_second;
    use units::length::millimeter;

    #[derive(Clone, Copy)]
    struct TestPort;

    struct TestStepperDevice {
        output: StepperVelocityEL70x1Output,
        input: StepperVelocityEL70x1Input,
    }

    impl TestStepperDevice {
        fn new() -> Self {
            Self {
                output: StepperVelocityEL70x1Output {
                    velocity: 0,
                    enable: false,
                    reduce_torque: false,
                    reset: false,
                    set_counter: None,
                },
                input: StepperVelocityEL70x1Input {
                    counter_value: 0,
                    ready_to_enable: true,
                    ready: true,
                    warning: false,
                    error: false,
                    moving_positive: false,
                    moving_negative: false,
                    torque_reduced: false,
                },
            }
        }
    }

    impl StepperVelocityEL70x1Device<TestPort> for TestStepperDevice {
        fn set_output(
            &mut self,
            _port: TestPort,
            value: StepperVelocityEL70x1Output,
        ) -> Result<(), anyhow::Error> {
            self.output = value;
            Ok(())
        }

        fn get_input(&self, _port: TestPort) -> Result<StepperVelocityEL70x1Input, anyhow::Error> {
            Ok(self.input.clone())
        }

        fn get_output(
            &self,
            _port: TestPort,
        ) -> Result<StepperVelocityEL70x1Output, anyhow::Error> {
            Ok(self.output.clone())
        }

        fn get_speed_range(&self, _port: TestPort) -> EL70x1SpeedRange {
            EL70x1SpeedRange::Steps1000
        }
    }

    fn make_test_motor() -> StepperVelocityEL70x1 {
        let device = Arc::new(RwLock::new(TestStepperDevice::new()));
        StepperVelocityEL70x1::new(device, TestPort)
    }

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

    #[test]
    fn test_manual_homing_stops_and_disables_on_endstop() {
        let mut controller = AddonMotorController::new(200);
        let mut motor = make_test_motor();
        let puller_velocity = AngularVelocity::new::<revolution_per_second>(10.0);

        controller.start_manual_homing();
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(false),
            Length::new::<millimeter>(0.0),
        );
        assert!(motor.is_enabled());
        assert!(motor.get_speed() > 0);
        assert_eq!(controller.get_pattern_state(), PatternControlState::Homing);

        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(0.0),
        );
        assert!(!motor.is_enabled());
        assert!(!controller.is_enabled());
        assert_eq!(controller.get_pattern_state(), PatternControlState::Idle);
    }

    #[test]
    fn test_pattern_requires_sensor_clear_before_pausing_after_kontur() {
        let mut controller = AddonMotorController::new(200);
        let mut motor = make_test_motor();
        let puller_velocity = AngularVelocity::new::<revolution_per_second>(10.0);
        controller.set_konturlaenge_mm(5.0);
        controller.set_pause_mm(3.0);
        controller.set_enabled(true);

        assert_eq!(controller.get_pattern_state(), PatternControlState::Homing);

        // Homing completes when endstop is hit once.
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(0.0),
        );
        assert_eq!(controller.get_pattern_state(), PatternControlState::Running);

        // Reaching kontur while sensor stays true does not pause immediately.
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(6.0),
        );
        assert_eq!(controller.get_pattern_state(), PatternControlState::Running);

        // Once sensor clears after kontur, next true transitions to Paused.
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(false),
            Length::new::<millimeter>(0.1),
        );
        assert_eq!(controller.get_pattern_state(), PatternControlState::Running);
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(0.1),
        );
        assert_eq!(controller.get_pattern_state(), PatternControlState::Paused);
        assert_eq!(motor.get_speed(), 0);
    }

    #[test]
    fn test_pattern_paused_transitions_back_to_running_after_pause_distance() {
        let mut controller = AddonMotorController::new(200);
        let mut motor = make_test_motor();
        let puller_velocity = AngularVelocity::new::<revolution_per_second>(10.0);
        controller.set_konturlaenge_mm(2.0);
        controller.set_pause_mm(3.0);
        controller.set_enabled(true);

        // Reach Paused deterministically.
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(0.0),
        ); // Homing -> Running
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(2.1),
        );
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(false),
            Length::new::<millimeter>(0.1),
        );
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(0.1),
        ); // Running -> Paused
        assert_eq!(controller.get_pattern_state(), PatternControlState::Paused);

        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(2.0),
        );
        assert_eq!(controller.get_pattern_state(), PatternControlState::Paused);

        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(1.1),
        );
        assert_eq!(controller.get_pattern_state(), PatternControlState::Running);
    }

    #[test]
    fn test_on_safety_stop_rearms_homing_when_pattern_mode_enabled() {
        let mut controller = AddonMotorController::new(200);
        let mut motor = make_test_motor();
        let puller_velocity = AngularVelocity::new::<revolution_per_second>(10.0);
        controller.set_konturlaenge_mm(2.0);
        controller.set_pause_mm(3.0);
        controller.set_enabled(true);

        // Reach Paused so safety-stop reset has meaningful state to clear.
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(0.0),
        ); // Homing -> Running
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(2.1),
        );
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(false),
            Length::new::<millimeter>(0.1),
        );
        controller.sync_motor_speed(
            &mut motor,
            puller_velocity,
            Some(true),
            Length::new::<millimeter>(0.1),
        ); // Running -> Paused
        assert_eq!(controller.get_pattern_state(), PatternControlState::Paused);

        controller.on_safety_stop();
        assert!(controller.is_enabled());
        assert_eq!(controller.get_pattern_state(), PatternControlState::Homing);
    }

    #[test]
    fn test_on_safety_stop_keeps_constant_mode_idle() {
        let mut controller = AddonMotorController::new(200);
        controller.set_enabled(true);
        assert_eq!(controller.get_pattern_state(), PatternControlState::Idle);

        controller.on_safety_stop();
        assert!(controller.is_enabled());
        assert_eq!(controller.get_pattern_state(), PatternControlState::Idle);
    }

    #[test]
    fn test_steps_to_reference_rpm_removes_ratio_scaling() {
        let mut controller = AddonMotorController::new(200);
        controller.set_master_ratio(2.0);
        controller.set_slave_ratio(1.0);

        // 1000 steps/s @ 200 steps/rev = 5 rev/s = 300 rpm motor speed.
        assert_eq!(controller.steps_to_rpm(1000), 300.0);
        // Ratio 2:1 => motor runs at 0.5x base speed, so reference rpm is 600.
        assert_eq!(controller.steps_to_reference_rpm(1000), 600.0);
    }

    #[test]
    fn test_steps_to_reference_rpm_with_higher_slave_ratio() {
        let mut controller = AddonMotorController::new(200);
        controller.set_master_ratio(1.0);
        controller.set_slave_ratio(2.0);

        // Ratio 1:2 => motor runs at 2x base speed, so reference rpm is halved.
        assert_eq!(controller.steps_to_reference_rpm(1000), 150.0);
    }
}
