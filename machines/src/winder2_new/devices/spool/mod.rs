use std::time::Instant;

use units::AngularVelocity;
use control_core::converters::angular_step_converter::AngularStepConverter;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;

use crate::types::Direction;
use crate::winder2_new::devices::{ Puller, TensionArm };

mod speed_controller;
use speed_controller::{SpeedController, AdaptiveSpeedController, MinMaxSpeedController};

mod types;
use types::SpeedControllers;
pub use types::SpeedControlMode as SpoolSpeedControlMode;

/// Represents the spool motor
#[derive(Debug)]
pub struct Spool
{
    hardware_interface: StepperVelocityEL70x1,
    direction:          Direction,
    speed_controllers:  SpeedControllers,
    speed_control_mode: SpeedControlMode,
    step_converter:     AngularStepConverter,
}

// getter + setter
impl Spool
{
    pub fn speed_control_mode(&self) -> SpeedControlMode
    {
        self.speed_control_mode
    }

    pub fn set_speed_control_mode(&mut self, value: SpeedControlMode)
    {
        if self.speed_control_mode == value { return; }

        // grab speed from active speed controller
        let current_speed = self.active_controller().speed();

        // change active speed controller
        self.speed_control_mode = value;

        // get active speed controller
        let controller = self.active_controller_mut();

        // Set the speed in the target controller and reset it for smooth transition
        controller.set_speed(current_speed);
        controller.reset();
        controller.set_speed(current_speed);
    }

    pub fn speed(&self) -> AngularVelocity 
    {
        self.active_controller().speed()
    }

    pub fn set_speed(&mut self, value: AngularVelocity) 
    {
        self.active_controller_mut().set_speed(value);
    }
}

// public interface
impl Spool
{
    pub fn new(hardware_interface: StepperVelocityEL70x1) -> Self
    {
        Self { 
            hardware_interface, 
            direction:          Direction::Forward, 
            speed_controllers:  SpeedControllers::new(),
            speed_control_mode: SpeedControlMode::Adaptive,
            step_converter:     AngularStepConverter::new(200),
        }
    }

    pub fn update(&mut self, t: Instant,tension_arm: &TensionArm, puller: &Puller)
    {
        let velocity = self.active_controller_mut().update_speed(t, tension_arm, puller);

        let velocity = if self.direction == Direction::Forward { velocity } else { -velocity };

        let steps_per_second = self.step_converter.angular_velocity_to_steps(velocity);

        self.hardware_interface.set_speed(steps_per_second);
    }
}

// utils
impl Spool
{
    fn active_controller(&self) -> &dyn SpeedController
    {
        use SpeedControlMode::*;

        match self.speed_control_mode
        {
            Adaptive => &self.speed_controllers.adaptive,
            MinMax   => &self.speed_controllers.minmax,
        }
    }

    fn active_controller_mut(&mut self) -> &mut dyn SpeedController
    {
        use SpeedControlMode::*;

        match self.speed_control_mode
        {
            Adaptive => &mut self.speed_controllers.adaptive,
            MinMax   => &mut self.speed_controllers.minmax,
        }
    }
}