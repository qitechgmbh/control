use std::time::Instant;

use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;

use crate::types::Direction;

/// Represents the puller motor
pub struct Spool
{
    hardware_interface: StepperVelocityEL70x1,

    direction: Direction,
}

impl Spool
{
    pub fn new(hardware_interface: StepperVelocityEL70x1) -> Self
    {
        todo!()
    }

    pub fn update(&mut self, t: Instant)
    {
        
    }
}