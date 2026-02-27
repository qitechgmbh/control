use std::time::Instant;

use units::{
    ConstZero,
    acceleration::meter_per_minute_per_second,
    f64::*,
    jerk::meter_per_minute_per_second_squared,
    length::centimeter,
    velocity::meter_per_minute,
};

use control_core::{
    controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    converters::linear_step_converter::LinearStepConverter,
};

use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;

mod types;
pub use types::GearRatio;
pub use types::State;
pub use types::Direction;

/// Represents the puller motor
#[derive(Debug)]
pub struct Puller
{
    hardware_interface: StepperVelocityEL70x1,

    state:      State,
    direction:  Direction,
    gear_ratio: GearRatio,

    /// actual speed of the motor
    current_speed: Velocity,
    target_speed:  Velocity,

    step_converter:          LinearStepConverter,
    acceleration_controller: LinearJerkSpeedController,
}

impl Puller
{
    const STEPS_PER_REVOLUTION:   i16 = 200;  // in steps
    const WHEEL_DIAMETER:         f64 = 8.0;  // in cm
    const MAX_MOTOR_SPEED:        f64 = 50.0; // in meters per minute
    const MAX_MOTOR_ACCELERATION: f64 = 5.0;  // in meters per minute per second
    const MAX_MOTOR_JERK:         f64 = 10.0; // in meters per minute per second squared

    pub fn new(hardware_interface: StepperVelocityEL70x1) -> Self
    {
        let step_converter = Self::create_step_converter();
        let acceleration_controller = Self::acceleration_controller();

        Self {
            // config
            state:        State::Disabled,
            direction:    Direction::Forward,
            gear_ratio:   GearRatio::OneToOne,
            target_speed: Velocity::ZERO,

            // other
            hardware_interface,
            step_converter,
            acceleration_controller,
            current_speed: Velocity::ZERO,
        }
    }

    // getters
    pub fn target_speed(&self) -> Velocity 
    { 
        self.target_speed 
    }

    pub fn motor_speed(&self) -> Velocity 
    { 
        self.current_speed 
    }

    pub fn output_speed(&self) -> Velocity 
    { 
        self.current_speed / self.gear_ratio.multiplier() 
    }

    pub fn state(&self) -> State 
    { 
        self.state 
    }

    pub fn direction(&self) -> Direction 
    { 
        self.direction 
    }

    pub fn gear_ratio(&self) -> GearRatio 
    { 
        self.gear_ratio 
    }

    // setters
    pub fn set_state(&mut self, value: State)
    {   
        self.state = value;
    }

    pub fn set_target_speed(&mut self, value: Velocity)
    {
        self.target_speed = value;
    }

    pub fn set_direction(&mut self, value: Direction)
    {
        self.direction = value;
    }

    pub fn set_gear_ratio(&mut self, value: GearRatio)
    {
        self.gear_ratio = value;
    }

    // functions
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
        self.hardware_interface.set_speed(steps_per_second);
    }

    // utils for new()
    fn create_step_converter() -> LinearStepConverter
    {
        let diameter = Length::new::<centimeter>(Self::WHEEL_DIAMETER);
        LinearStepConverter::from_diameter(Self::STEPS_PER_REVOLUTION, diameter)
    }

    fn acceleration_controller() -> LinearJerkSpeedController
    {
        let acceleration 
            = Acceleration::new::<meter_per_minute_per_second>(Self::MAX_MOTOR_ACCELERATION);

        let jerk         
            = Jerk::new::<meter_per_minute_per_second_squared>(Self::MAX_MOTOR_JERK);

        let speed        
            = Velocity::new::<meter_per_minute>(Self::MAX_MOTOR_SPEED);

        LinearJerkSpeedController::new_simple(Some(speed),acceleration, jerk)
    }
}