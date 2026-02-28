use std::time::Instant;

use units::{ConstZero, Length, Velocity};
use units::length::meter;
use units::velocity::meter_per_second;

#[derive(Debug)]
pub struct SpoolLengthTask
{
    current_length: Length,
    target_length:  Length,
    last_check:     Instant,
}

impl SpoolLengthTask
{
    pub fn new() -> Self
    {
        SpoolLengthTask {
            target_length:  Length::new::<meter>(0.0),
            current_length: Length::new::<meter>(0.0),
            last_check:     Instant::now(),
        }
    }

    pub fn is_complete(&self) -> bool
    {
        self.current_length >= self.target_length
    }

    pub const fn current_length(&self) -> Length {
        self.current_length
    }

    pub const fn target_length(&self) -> Length {
        self.target_length
    }

    pub fn set_target_length(&mut self, target_length: Length) {
        self.target_length = target_length;
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
        self.current_length += meters_pulled_this_interval.abs();
        self.last_check = now;
    }

    pub fn reset(&mut self, now: Instant)
    {
        self.current_length = Length::ZERO;
        self.last_check = now;
    }
}