use std::time::Instant;

use serde::{Deserialize, Serialize};
use units::{ConstZero, Length, length::meter, velocity::meter_per_second};

use crate::winder2::{Mode as Winder2Mode, devices::Puller};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Mode
{
    NoAction,
    Pull,
    Hold,
}

#[derive(Debug)]
pub struct AutomaticAction 
{
    mode: Mode,
    progress: Length,
    progress_last_check: Instant,
    target_length: Length,
}

impl Default for AutomaticAction 
{
    fn default() -> Self {
        AutomaticAction {
            progress: Length::new::<meter>(0.0),
            progress_last_check: Instant::now(),
            target_length: Length::new::<meter>(0.0),
            mode: Mode::NoAction,
        }
    }
}

impl AutomaticAction
{
    pub const fn reset(&mut self, now: Instant) 
    {
        self.progress = Length::ZERO;
        self.progress_last_check = now;
    }

    pub fn calculate_progress(&mut self, now: Instant, puller: &Puller) 
    {
        // Calculate time elapsed since last progress check (in minutes)

        let dt = now
            .duration_since(self.progress_last_check)
            .as_secs_f64();

        // Calculate distance pulled during this time interval
        let meters_pulled_this_interval = Length::new::<meter>(
            puller.output_speed().get::<meter_per_second>() * dt,
        );

        // Update total meters pulled
        self.progress += meters_pulled_this_interval.abs();
        self.progress_last_check = now;
    }

    pub fn update(
        &mut self, 
        now: Instant, 
        winder_mode: Winder2Mode, 
        puller: &Puller
    ) -> Option<Winder2Mode>
    {
        if self.mode == Mode::NoAction
        {
            self.calculate_progress(now, puller);
        }

        if winder_mode != Winder2Mode::Pull && winder_mode != Winder2Mode::Wind
        {
            self.progress_last_check = now;
            return None;
        }

        self.calculate_progress(now, puller);

        if self.progress >= self.target_length 
        {
            match self.mode 
            {
                Mode::NoAction => (),
                Mode::Pull => {
                    self.reset(now);
                    return Some(Winder2Mode::Pull);
                }
                Mode::Hold => {
                    self.reset(now);
                    return Some(Winder2Mode::Hold);
                }
            }
        }

        None
    }
}