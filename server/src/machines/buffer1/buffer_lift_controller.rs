use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;

#[derive(Debug)]
pub struct BufferTowerController {
    enabled: bool,
    /// Stepper driver. Controls buffer stepper motor
    pub stepper_driver: StepperVelocityEL70x1,
}

impl BufferTowerController {
    pub const fn new(driver: StepperVelocityEL70x1) -> Self {
        Self {
            enabled: false,
            stepper_driver: driver,
        }
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            self.stepper_driver.set_enabled(true);
            let _ = self.stepper_driver.set_speed(10.0);
        } else {
            self.stepper_driver.set_enabled(false);
            let _ = self.stepper_driver.set_speed(0.0);
        }
    }
}
