use std::time::Instant;

use control_core::controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController;
use serde::{Deserialize, Serialize};
use units::{Acceleration, ConstZero, Jerk, Velocity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode 
{
    Fixed,
    Adaptive,
}

#[derive(Debug)]
pub struct SpeedController
{
    // config
    mode: Mode,
    enabled: bool,
    multiplier: f64,

    // state
    speed: Velocity,

    strategies: Strategies,
    acceleration_controller: LinearJerkSpeedController,
}

impl SpeedController
{
    pub fn new(speed_max: Velocity, jerk_max: Jerk, acceleration_max: Acceleration) -> Self
    {
        let fixed = FixedSpeedStrategy { 
            target_speed: Velocity::ZERO 
        };

        let adaptive = AdaptiveSpeedStrategy { 
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
            mode:       Mode::Fixed, 
            enabled:    false,
            multiplier: 1.0,
            speed:      Velocity::ZERO,
            strategies: Strategies { fixed, adaptive },
            acceleration_controller, 
        }
    }

    pub fn update(&mut self, t: Instant)
    {
        use Mode::*;

        let speed = match self.enabled 
        {
            true => match self.mode 
            {
                Fixed     => self.strategies.fixed.compute(),
                Adaptive => self.strategies.adaptive.compute(),
            },
            false => Velocity::ZERO,
        };

        // Apply acceleration control
        self.speed = self.acceleration_controller.update(speed * self.multiplier, t);
    }

    pub fn mode(&self) -> Mode
    {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode)
    {
        self.mode = mode;
    }

    pub fn set_enabled(&mut self, value: bool)
    {
        self.enabled = value;
    }

    pub fn speed(&self) -> Velocity
    {
        self.speed
    }

    pub fn set_multiplier(&mut self, multiplier: f64)
    {
        self.multiplier = multiplier;
    }

    pub fn strategies(&self) -> &Strategies
    {
        &self.strategies
    }

    pub fn strategies_mut(&mut self) -> &mut Strategies
    {
        &mut self.strategies
    }
}

#[derive(Debug)]
pub struct Strategies
{
    pub fixed:    FixedSpeedStrategy,
    pub adaptive: AdaptiveSpeedStrategy,
}

#[derive(Debug)]
pub struct FixedSpeedStrategy
{
    target_speed: Velocity,
}

impl FixedSpeedStrategy
{
    pub fn compute(&self) -> Velocity
    {
        self.target_speed
    }

    pub fn target_speed(&self) -> Velocity {
        self.target_speed
    }

    pub fn set_target_speed(&mut self, speed: Velocity) {
        self.target_speed = speed;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AdaptiveSpeedStrategy {
    base_speed: Velocity,
    deviation_max: Velocity,
    modulation: f64, // (-1.0 to 1.0)
}

impl AdaptiveSpeedStrategy
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

    pub fn set_modulation(&mut self, modulation: f64) {
        self.modulation = modulation.clamp(-1.0, 1.0);
    }
}