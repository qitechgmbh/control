use std::time::Instant;

use serde::{Deserialize, Serialize};

use units::{ConstZero, Length, Velocity};
use units::length::meter;
use units::velocity::meter_per_second;

use ethercat_hal::io::{
    analog_input::AnalogInput, 
    digital_input::DigitalInput, 
    digital_output::DigitalOutput, 
    stepper_velocity_el70x1::StepperVelocityEL70x1
};

#[derive(Debug)]
pub struct Hardware
{
    pub spool_motor:            StepperVelocityEL70x1,
    pub puller_motor:           StepperVelocityEL70x1,
    pub traverse_motor:         StepperVelocityEL70x1,
    pub traverse_limit_switch:  DigitalInput,
    pub traverse_laser_pointer: DigitalOutput,
    pub tension_arm_sensor:     AnalogInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode 
{
    Standby,
    Hold,
    Pull,
    Wind,
}

#[derive(Debug)]
pub struct SpoolLengthTask
{
    progress:   Length,
    target:     Length,
    last_check: Instant,
}

impl SpoolLengthTask
{
    pub fn new() -> Self
    {
        SpoolLengthTask {
            target:     Length::new::<meter>(0.0),
            progress:   Length::new::<meter>(0.0),
            last_check: Instant::now(),
        }
    }

    pub fn is_complete(&self) -> bool
    {
        self.progress >= self.target
    }

    pub const fn progress(&self) -> Length 
    {
        self.progress
    }

    pub const fn target_length(&self) -> Length 
    {
        self.target
    }

    pub fn set_target_length(&mut self, target_length: Length) 
    {
        self.target = target_length;
    }

    pub fn update_timer(&mut self, now: Instant)
    {
        self.last_check = now;
    }

    pub fn update_progress(&mut self, now: Instant, velocity: Velocity) 
    {
        // Calculate time elapsed since last progress check (in minutes)
        let dt = now.duration_since(self.last_check).as_secs_f64();

        // Calculate distance pulled during this time interval
        let meters_pulled_this_interval = Length::new::<meter>(
            velocity.get::<meter_per_second>() * dt,
        );

        // Update total meters pulled
        self.progress += meters_pulled_this_interval.abs();
        self.last_check = now;
    }

    pub fn reset(&mut self, now: Instant)
    {
        self.progress = Length::ZERO;
        self.last_check = now;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpoolLengthTaskCompletedAction
{
    NoAction,
    Pull,
    Hold,
}