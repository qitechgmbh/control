use control_core::converters::angular_step_converter::AngularStepConverter;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use units::f64::AngularVelocity;
use units::{
    Length, angular_velocity::revolution_per_minute, angular_velocity::revolution_per_second,
};

/// Ratio-based puller follower without pattern/homing (steppers 4 and 5).
#[derive(Debug)]
pub struct RatioFollowMotor {
    enabled: bool,
    forward: bool,
    master_ratio: f64,
    slave_ratio: f64,
    converter: AngularStepConverter,
}

impl RatioFollowMotor {
    pub fn new(steps_per_revolution: i16) -> Self {
        Self {
            enabled: false,
            forward: true,
            master_ratio: 1.0,
            slave_ratio: 1.0,
            converter: AngularStepConverter::new(steps_per_revolution),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_forward(&mut self, forward: bool) {
        self.forward = forward;
    }

    pub const fn is_forward(&self) -> bool {
        self.forward
    }

    pub fn set_master_ratio(&mut self, master: f64) {
        self.master_ratio = master.max(0.1);
    }

    pub fn set_slave_ratio(&mut self, slave: f64) {
        self.slave_ratio = slave.max(0.1);
    }

    pub const fn get_master_ratio(&self) -> f64 {
        self.master_ratio
    }

    pub const fn get_slave_ratio(&self) -> f64 {
        self.slave_ratio
    }

    pub fn on_safety_stop(&mut self) {
        // Ratio followers have no pattern state to reset.
    }

    pub fn steps_to_rpm(&self, steps: i32) -> f64 {
        self.converter
            .steps_to_angular_velocity(steps as f64)
            .get::<revolution_per_minute>()
            .abs()
    }

    pub fn steps_to_reference_rpm(&self, steps: i32) -> f64 {
        let motor_rpm = self.steps_to_rpm(steps);
        let ratio = (self.slave_ratio / self.master_ratio).abs();
        if ratio > f64::EPSILON {
            motor_rpm / ratio
        } else {
            motor_rpm
        }
    }

    fn calculate_motor_velocity(
        &self,
        puller_angular_velocity: AngularVelocity,
    ) -> AngularVelocity {
        if !self.enabled {
            return AngularVelocity::new::<revolution_per_second>(0.0);
        }
        let ratio = self.slave_ratio / self.master_ratio;
        let velocity = puller_angular_velocity * ratio;
        if self.forward { velocity } else { -velocity }
    }

    pub fn sync_motor_speed(
        &mut self,
        motor: &mut StepperVelocityEL70x1,
        puller_angular_velocity: AngularVelocity,
        _endstop_hit: Option<bool>,
        _puller_length_moved: Length,
    ) {
        if !self.enabled {
            if motor.is_enabled() {
                motor.set_enabled(false);
            }
            let _ = motor.set_speed(0.0);
            return;
        }

        if !motor.is_enabled() {
            motor.set_enabled(true);
        }

        let target_velocity = self.calculate_motor_velocity(puller_angular_velocity);
        let steps_per_second = self.converter.angular_velocity_to_steps(target_velocity);
        let _ = motor.set_speed(steps_per_second);
    }
}
