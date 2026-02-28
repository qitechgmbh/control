use ethercat_hal::io::{
    analog_input::AnalogInput, 
    digital_input::DigitalInput, 
    digital_output::DigitalOutput, 
    stepper_velocity_el70x1::StepperVelocityEL70x1
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode 
{
    Standby,
    Hold,
    Pull,
    Wind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpoolLengthTaskCompletedAction
{
    NoAction,
    Pull,
    Hold,
}

#[derive(Debug)]
pub struct Hardware
{
    pub spool_motor:            StepperVelocityEL70x1,
    pub traverse_motor:         StepperVelocityEL70x1,
    pub traverse_limit_switch:  DigitalInput,
    pub traverse_laser_pointer: DigitalOutput,
    pub puller_motor:           StepperVelocityEL70x1,
    pub tension_arm_sensor:     AnalogInput,
}