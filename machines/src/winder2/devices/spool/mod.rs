use std::time::Instant;

use serde::{Deserialize, Serialize};
use units::AngularVelocity;
use control_core::converters::angular_step_converter::AngularStepConverter;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;

use crate::types::Direction;
use crate::winder2::devices::{ Puller, TensionArm };

use super::OperationState;

mod speed_controller;
use speed_controller::{SpeedController, AdaptiveSpeedController, MinMaxSpeedController};

/// Represents the spool motor
#[derive(Debug)]
pub struct Spool
{
    motor:              StepperVelocityEL70x1,
    operation_state:              OperationState,
    direction:          Direction,
    speed_control_mode: SpeedControlMode,
    step_converter:     AngularStepConverter,

    pub speed_controllers: SpeedControllers,
}

// public interface
impl Spool
{
    pub fn new(motor: StepperVelocityEL70x1) -> Self
    {
        Self { 
            motor, 
            operation_state:              OperationState::Disabled,
            direction:          Direction::Forward, 
            speed_controllers:  SpeedControllers::new(),
            speed_control_mode: SpeedControlMode::Adaptive,
            step_converter:     AngularStepConverter::new(200),
        }
    }

    pub fn update(&mut self, t: Instant,tension_arm: &TensionArm, puller: &Puller)
    {
        let multiplier = if self.direction == Direction::Forward { 1.0 } else { -1.0 };

        let controller = self.active_controller_mut();

        let velocity = controller.update_speed(t, multiplier, tension_arm, puller);

        let steps_per_second = self.step_converter.angular_velocity_to_steps(velocity);

        _ = self.motor.set_speed(steps_per_second);
    }
}

// getter + setter
impl Spool
{
    pub fn speed_control_mode(&self) -> SpeedControlMode
    {
        self.speed_control_mode
    }

    pub fn set_speed_control_mode(&mut self, mode: SpeedControlMode)
    {
        if self.speed_control_mode == mode { return; }

        self.speed_control_mode = mode;

        // get active speed controller
        let controller = self.active_controller_mut();
        let current_speed = controller.speed();

        // Set the speed in the target controller and reset it for smooth transition
        controller.set_speed(current_speed);
        controller.reset();
        controller.set_speed(current_speed);
    }

    pub fn speed(&self) -> AngularVelocity 
    {
        self.active_controller().speed()
    }

    pub fn set_operation_state(&mut self, operation_state: OperationState)
    {
        use OperationState::*;

        // No change, nothing to do
        if self.operation_state == operation_state { return; }

        // Leaving disabled state, enable motor
        if self.operation_state == Disabled {
            self.motor.set_enabled(true);
        }

        self.active_controller_mut().set_enabled(operation_state == Running);
        self.operation_state = operation_state;
    }

    pub fn direction(&self) -> Direction
    {
        self.direction
    }

    pub fn set_direction(&mut self, direction: Direction)
    {
        self.direction = direction;
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

// other types 
#[derive(Debug)]
pub struct SpeedControllers
{
    pub minmax:   MinMaxSpeedController,
    pub adaptive: AdaptiveSpeedController,
}

impl SpeedControllers
{
    pub fn new() -> Self
    {
        Self { 
            minmax: MinMaxSpeedController::new(), 
            adaptive: AdaptiveSpeedController::new() 
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeedControlMode
{
    Adaptive,
    MinMax,
}