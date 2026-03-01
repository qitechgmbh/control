use std::time::Instant;

use serde::{Deserialize, Serialize};
use units::ConstZero;
use units::{
    acceleration::meter_per_minute_per_second,
    f64::*,
    jerk::meter_per_minute_per_second_squared,
    length::centimeter,
    velocity::meter_per_minute,
};

use meter_per_minute as velocity_unit;
use meter_per_minute_per_second as acceleration_unit;
use meter_per_minute_per_second_squared as jerk_unit;

use control_core::{ converters::linear_step_converter::LinearStepConverter };

use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;

use crate::types::Direction;

use super::OperationState;

mod speed_controller;
pub use speed_controller::SpeedController;

pub use speed_controller::{
    Mode as SpeedControlMode,
    Strategies as SpeedControllerStrategies
};

/// Represents the puller motor
#[derive(Debug)]
pub struct Puller
{
    // hardware
    motor: StepperVelocityEL70x1,

    // config
    operation_state:      OperationState,
    direction:  Direction,
    gear_ratio: GearRatio,

    // misc
    speed_controller: SpeedController,
    step_converter:   LinearStepConverter,
}

// constants
impl Puller
{
    // fixed
    const STEPS_PER_REVOLUTION:   i16 = 200;  // in steps
    const WHEEL_DIAMETER:         f64 = 8.0;  // in cm
    const MOTOR_SPEED_MAX:        f64 = 50.0; // in meters per minute
    const MOTOR_ACCELERATION_MAX: f64 = 5.0;  // in meters per minute per second
    const MOTOR_JERK_MAX:         f64 = 10.0; // in meters per minute per second squared
}

// public interface
impl Puller
{
    pub fn new(motor: StepperVelocityEL70x1) -> Self
    {
        let speed_controller = {
            let speed_max = Velocity::new::<velocity_unit>(Self::MOTOR_SPEED_MAX);
            let acc_max   = Acceleration::new::<acceleration_unit>(Self::MOTOR_ACCELERATION_MAX);
            let jerk_max  = Jerk::new::<jerk_unit>(Self::MOTOR_JERK_MAX);
            SpeedController::new(speed_max, jerk_max, acc_max)
        };

        let step_converter = {
            let diameter = Length::new::<centimeter>(Self::WHEEL_DIAMETER);
            LinearStepConverter::from_diameter(Self::STEPS_PER_REVOLUTION, diameter)
        };

        Self {
            // config
            operation_state:      OperationState::Disabled,
            direction:  Direction::Forward,
            gear_ratio: GearRatio::OneToOne,

            // misc
            motor,
            speed_controller,
            step_converter,
        }
    }

    pub fn update(&mut self, t: Instant) 
    {
        // update speed
        self.speed_controller.update(t);

        // retrieve speed
        let speed = self.speed_controller.speed();

        // only disable motor once it reaches a really low speed
        // to prevent an abrupt stop
        if self.operation_state == OperationState::Disabled 
            && speed.get::<meter_per_minute>() <= 0.1
        {
            self.motor.set_enabled(false);
        }

        // convert to steps/sec
        let steps_per_second = self.step_converter.velocity_to_steps(speed);

        // write to hardware
        _ = self.motor.set_speed(steps_per_second);
    }
}

// getter + setter
impl Puller
{
    #[allow(dead_code)]
    pub fn operation_state(&self) -> OperationState 
    { 
        self.operation_state 
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

        self.speed_controller.set_enabled(operation_state == Running);
        self.operation_state = operation_state;
    }

    pub fn direction(&self) -> Direction 
    { 
        self.direction 
    }

    pub fn set_direction(&mut self, direction: Direction)
    {
        self.direction = direction;
        self.update_multiplier();
    }

    pub fn gear_ratio(&self) -> GearRatio 
    { 
        self.gear_ratio 
    }

    pub fn set_gear_ratio(&mut self, gear_ratio: GearRatio)
    {
        self.gear_ratio = gear_ratio;
        self.update_multiplier();

        // reset configured speeds when gear ratio changes
        // for added safety, since changing gear ratio should
        // require a reconfiguration of the machine
        let strategies = self.speed_controller.strategies_mut();
        strategies.fixed.set_target_speed(Velocity::ZERO);
        strategies.adaptive.set_base_speed(Velocity::ZERO);
        strategies.adaptive.set_deviation_max(Velocity::ZERO);
    }

    pub fn output_speed(&self) -> Velocity 
    { 
        self.speed_controller.speed() / self.gear_ratio.multiplier() 
    }

    pub fn speed_control_mode(&self) -> SpeedControlMode
    {
        self.speed_controller.mode()
    }

    pub fn set_speed_control_mode(&mut self, mode: SpeedControlMode)
    {
        self.speed_controller.set_mode(mode);
    }

    pub fn speed_controller_strategies(&self) -> &SpeedControllerStrategies
    {
        self.speed_controller.strategies()
    }

    pub fn speed_controller_strategies_mut(&mut self) -> &mut SpeedControllerStrategies
    {
        self.speed_controller.strategies_mut()
    }
}

// utils
impl Puller
{
    fn update_multiplier(&mut self)
    {
        let multiplier = self.gear_ratio.multiplier();
        let multiplier = match self.direction 
        {
            Direction::Forward => multiplier,
            Direction::Reverse => -multiplier,
        };

        self.speed_controller.set_multiplier(multiplier);
    }
}

// other types

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GearRatio 
{
    OneToOne,
    FiveToOne,
    TenToOne,
}

impl GearRatio 
{
    /// Get the speed multiplier for this gear ratio
    pub fn multiplier(&self) -> f64 
    {
        match self 
        {
            GearRatio::OneToOne  => 1.0,
            GearRatio::FiveToOne => 5.0,
            GearRatio::TenToOne  => 10.0,
        }
    }
}