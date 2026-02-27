use std::time::Instant;

use serde::{Deserialize, Serialize};
use units::{
    ConstZero,
    acceleration::meter_per_minute_per_second,
    f64::*,
    jerk::meter_per_minute_per_second_squared,
    length::{centimeter, millimeter},
    velocity::meter_per_minute,
};

use control_core::{
    controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    converters::linear_step_converter::LinearStepConverter,
};

use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;

use crate::types::Direction;

/// Represents the puller motor
#[derive(Debug)]
pub struct Puller
{
    motor: StepperVelocityEL70x1,

    state:      State,
    direction:  Direction,
    gear_ratio: GearRatio,
    regulation: SpeedRegulation,

    /// actual speed of the motor
    current_speed: Velocity,
    target_speed:  Velocity,
    target_diameter: Length,

    step_converter:          LinearStepConverter,
    acceleration_controller: LinearJerkSpeedController,
}

// constants
impl Puller
{
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
        let step_converter = Self::create_step_converter();
        let acceleration_controller = Self::create_acceleration_controller();

        Self {
            // config
            state:           State::Disabled,
            direction:       Direction::Forward,
            gear_ratio:      GearRatio::OneToOne,
            target_speed:    Velocity::new::<meter_per_minute>(1.0),
            target_diameter: Length::new::<millimeter>(1.75),
            regulation:      SpeedRegulation::Adapative,

            // values
            current_speed: Velocity::new::<meter_per_minute>(1.0),

            // misc
            motor,
            step_converter,
            acceleration_controller,
        }
    }

    pub fn update(&mut self, t: Instant) 
    {
        // Compute target motor speed considering gear ratio and direction
        let target_speed = self.target_speed * self.gear_ratio.multiplier();

        let target_speed = match self.direction 
        {
            Direction::Forward => target_speed,
            Direction::Reverse => -target_speed,
        };

        // Apply acceleration control
        let controlled_speed = self.acceleration_controller.update(target_speed, t);
        self.current_speed = controlled_speed;

        // Convert to steps/sec and write to hardware
        let steps_per_second = self.step_converter.velocity_to_steps(controlled_speed);
        self.motor.set_speed(steps_per_second);
    }
}

// getter + setter
impl Puller
{
    pub fn state(&self) -> State 
    { 
        self.state 
    }

    pub fn set_state(&mut self, value: State)
    {   
        self.state = value;
    }

    pub fn speed_regulation_mode(&self) -> SpeedRegulation 
    { 
        self.regulation 
    }

    pub fn set_speed_regulation(&mut self, value: SpeedRegulation)
    {   
        self.regulation = value;
    }

    pub fn direction(&self) -> Direction 
    { 
        self.direction 
    }

    pub fn set_direction(&mut self, value: Direction)
    {
        self.direction = value;
    }

    pub fn gear_ratio(&self) -> GearRatio 
    { 
        self.gear_ratio 
    }

    pub fn set_gear_ratio(&mut self, value: GearRatio)
    {
        self.gear_ratio = value;
    }

    pub fn target_speed(&self) -> Velocity
    { 
        self.target_speed 
    }

    pub fn set_target_speed(&mut self, value: Velocity)
    {
        self.target_speed = value;
    }

    pub fn target_diameter(&self) -> Length
    { 
        self.target_diameter 
    }

    pub fn set_target_diameter(&mut self, value: Length)
    {
        self.target_diameter = value;
    }

    pub fn motor_speed(&self) -> Velocity 
    { 
        self.current_speed 
    }

    pub fn output_speed(&self) -> Velocity 
    { 
        self.current_speed / self.gear_ratio.multiplier() 
    }
}

// utils
impl Puller
{
    fn create_step_converter() -> LinearStepConverter
    {
        let diameter = Length::new::<centimeter>(Self::WHEEL_DIAMETER);
        LinearStepConverter::from_diameter(Self::STEPS_PER_REVOLUTION, diameter)
    }

    fn create_acceleration_controller() -> LinearJerkSpeedController
    {
        let acceleration 
            = Acceleration::new::<meter_per_minute_per_second>(Self::MOTOR_ACCELERATION_MAX);

        let jerk         
            = Jerk::new::<meter_per_minute_per_second_squared>(Self::MOTOR_JERK_MAX);

        let speed        
            = Velocity::new::<meter_per_minute>(Self::MOTOR_SPEED_MAX);

        LinearJerkSpeedController::new_simple(Some(speed),acceleration, jerk)
    }
}

// other types

// state
#[derive(Debug,Clone, Copy, PartialEq, Eq)]
pub enum State 
{
    Disabled,
    Holding,
    Running,
}

// gear ratio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GearRatio 
{
    OneToOne,
    OneToFive,
    OneToTen,
}

impl GearRatio 
{
    /// Get the speed multiplier for this gear ratio
    pub fn multiplier(&self) -> f64 
    {
        match self 
        {
            GearRatio::OneToOne => 1.0,
            GearRatio::OneToFive => 5.0,
            GearRatio::OneToTen => 10.0,
        }
    }
}

// speed regulation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeedRegulation 
{
    Fixed,
    Adapative,
}