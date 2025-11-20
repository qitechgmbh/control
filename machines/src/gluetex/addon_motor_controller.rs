use control_core::converters::angular_step_converter::AngularStepConverter;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use units::angular_velocity::revolution_per_second;
use units::f64::AngularVelocity;

/// Controller for addon motors that follow the puller speed with a configurable ratio
#[derive(Debug)]
pub struct AddonMotorController {
    /// Whether the motor is enabled (Run mode)
    enabled: bool,
    /// Master ratio value (e.g., 2 in "2:1")
    master_ratio: f64,
    /// Slave ratio value (e.g., 1 in "2:1")
    slave_ratio: f64,
    /// Converter for angular velocity to steps
    converter: AngularStepConverter,
}

impl AddonMotorController {
    /// Create a new addon motor controller
    ///
    /// # Arguments
    /// * `steps_per_revolution` - Number of steps per revolution for the stepper motor
    pub fn new(steps_per_revolution: i16) -> Self {
        Self {
            enabled: false,
            master_ratio: 1.0,
            slave_ratio: 1.0,
            converter: AngularStepConverter::new(steps_per_revolution),
        }
    }

    /// Set whether the motor is enabled (Run mode)
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get whether the motor is enabled
    pub const fn is_enabled(&self) -> bool {
        self.enabled
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
        puller_angular_velocity * ratio
    }

    /// Update the motor speed based on the puller angular velocity
    ///
    /// # Arguments
    /// * `motor` - The stepper motor to control
    /// * `puller_angular_velocity` - The current angular velocity of the puller
    pub fn sync_motor_speed(
        &self,
        motor: &mut StepperVelocityEL70x1,
        puller_angular_velocity: AngularVelocity,
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

        // Calculate target velocity
        let target_velocity = self.calculate_motor_velocity(puller_angular_velocity);

        // Convert to steps per second and set motor speed
        let steps_per_second = self.converter.angular_velocity_to_steps(target_velocity);
        let _ = motor.set_speed(steps_per_second);
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
