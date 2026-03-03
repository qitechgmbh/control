use std::time::Instant;

use control_core::controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController;
use serde::{Deserialize, Serialize};
use units::{Acceleration, ConstZero, Jerk, Velocity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm 
{
    Fixed,
    Adaptive,
}

#[derive(Debug)]
pub struct SpeedController
{
    enabled:    bool,
    algorithm:  Algorithm,
    speed:      Velocity,
    algorithms: Algorithms,

    // currently all algorithms share acceleration controller
    // so we keep it in here. Might consider moving to each
    // algorithm and introduce a sort of sync function
    // for smooth transitions when switching algorithms
    acceleration_controller: LinearJerkSpeedController,
}

impl SpeedController
{
    pub fn new(speed_max: Velocity, jerk_max: Jerk, acceleration_max: Acceleration) -> Self
    {
        let fixed = FixedSpeedAlgorithm { 
            speed_max,
            target_speed: Velocity::ZERO,
        };

        let adaptive = AdaptiveSpeedAlgorithm { 
            speed_max,
            speed_base: Velocity::ZERO, 
            deviation_limit: Velocity::ZERO, 
            modulation: 0.0,
        };

        let acceleration_controller = LinearJerkSpeedController::new_simple(
            Some(speed_max), 
            acceleration_max, 
            jerk_max
        );

        Self { 
            algorithm:  Algorithm::Fixed, 
            enabled:    false,
            speed:      Velocity::ZERO,
            algorithms: Algorithms { fixed, adaptive },
            acceleration_controller, 
        }
    }

    pub fn update(&mut self, t: Instant, multiplier: f64)
    {
        use Algorithm::*;

        let speed = match self.enabled 
        {
            true => match self.algorithm 
            {
                Fixed    => self.algorithms.fixed.compute(),
                Adaptive => self.algorithms.adaptive.compute(),
            },
            false => Velocity::ZERO,
        };

        // Apply acceleration control
        self.speed = self.acceleration_controller.update(speed * multiplier, t);
    }

    pub fn active_algorithm(&self) -> Algorithm
    {
        self.algorithm
    }

    pub fn select_algorithm(&mut self, mode: Algorithm)
    {
        self.algorithm = mode;
    }

    pub fn set_enabled(&mut self, value: bool)
    {
        self.enabled = value;
    }

    pub fn speed(&self) -> Velocity
    {
        self.speed
    }

    pub fn algorithms(&self) -> &Algorithms
    {
        &self.algorithms
    }

    pub fn algorithms_mut(&mut self) -> &mut Algorithms
    {
        &mut self.algorithms
    }
}

#[derive(Debug)]
pub struct Algorithms
{
    pub fixed:    FixedSpeedAlgorithm,
    pub adaptive: AdaptiveSpeedAlgorithm,
}

#[derive(Debug)]
pub struct FixedSpeedAlgorithm
{
    target_speed: Velocity,
    speed_max:    Velocity,
}

impl FixedSpeedAlgorithm
{
    pub fn compute(&self) -> Velocity
    {
        self.target_speed
    }

    pub fn target_speed(&self) -> Velocity 
    {
        self.target_speed
    }

    pub fn set_target_speed(&mut self, speed: Velocity) 
    {
        self.target_speed = speed.min(self.speed_max);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AdaptiveSpeedAlgorithm 
{
    speed_max:  Velocity,
    speed_base: Velocity,
    deviation_limit: Velocity,
    modulation: f64, // (-1.0 to 1.0)
}

impl AdaptiveSpeedAlgorithm
{
    pub fn compute(&self) -> Velocity
    {
        self.speed_base + (self.deviation_limit * self.modulation)
    }

    pub fn speed_base(&self) -> Velocity {
        self.speed_base
    }

    pub fn set_speed_base(&mut self, speed: Velocity) {
        self.speed_base = speed
            .min(self.speed_max) // ensure < max
            .max(Velocity::ZERO) // ensure > 0
            ;

        // update deviation limit as well
        self.set_deviation_limit(self.deviation_limit);
    }

    pub fn deviation_limit(&self) -> Velocity {
        self.deviation_limit
    }

    pub fn set_deviation_limit(&mut self, deviation_limit: Velocity) 
    {
        self.deviation_limit = 
            // ensure > 0
            if deviation_limit < Velocity::ZERO
                { Velocity::ZERO }
            // ensure base - deviation can't below zero
            else if deviation_limit > self.speed_base 
                { self.speed_base } 
            // ensure base + deviation can't exceed max
            else if self.speed_base + deviation_limit > self.speed_max 
                { self.speed_max - self.speed_base }
            // in valid range
            else 
                { deviation_limit };
    }

    #[allow(dead_code)]
    pub fn set_modulation(&mut self, modulation: f64) 
    {
        self.modulation = modulation.clamp(-1.0, 1.0);
    }
}