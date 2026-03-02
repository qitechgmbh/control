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
            target_speed: Velocity::ZERO 
        };

        let adaptive = AdaptiveSpeedAlgorithm { 
            base_speed:    Velocity::ZERO, 
            deviation_max: Velocity::ZERO, 
            modulation:    0.0,
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
        self.target_speed = speed;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AdaptiveSpeedAlgorithm 
{
    base_speed: Velocity,
    deviation_max: Velocity,
    modulation: f64, // (-1.0 to 1.0)
}

impl AdaptiveSpeedAlgorithm
{
    pub fn compute(&self) -> Velocity
    {
        self.base_speed + (self.deviation_max * self.modulation)
    }

    pub fn base_speed(&self) -> Velocity {
        self.base_speed
    }

    pub fn set_base_speed(&mut self, speed: Velocity) {
        self.base_speed = speed;
    }

    pub fn deviation_max(&self) -> Velocity {
        self.deviation_max
    }

    pub fn set_deviation_max(&mut self, deviation: Velocity) {
        self.deviation_max = deviation;
    }

    #[allow(dead_code)]
    pub fn set_modulation(&mut self, modulation: f64) 
    {
        self.modulation = modulation.clamp(-1.0, 1.0);
    }
}