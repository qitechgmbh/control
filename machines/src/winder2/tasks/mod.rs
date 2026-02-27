use std::time::Instant;

use serde::{Deserialize, Serialize};
use units::{ConstZero, Length, length::meter, velocity::meter_per_second};

use crate::winder2::devices::Puller;

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

    pub fn reset(&mut self, now: Instant) 
    {
        self.progress = Length::ZERO;
        self.progress_last_check = now;
    }

    pub fn is_complete(&self) -> bool
    {
        self.progress >= self.target_length
    }

    pub fn update_timer(&mut self, now: Instant)
    {
        self.last_check = now;
    }

    pub fn update_progress(&mut self, now: Instant, puller: &Puller) 
    {
        // Calculate time elapsed since last progress check (in minutes)
        let dt = now.duration_since(self.progress_last_check).as_secs_f64();

        // Calculate distance pulled during this time interval
        let meters_pulled_this_interval = Length::new::<meter>(
            puller.output_speed().get::<meter_per_second>() * dt,
        );

        // Update total meters pulled
        self.progress += meters_pulled_this_interval.abs();
        self.progress_last_check = now;
    }
}