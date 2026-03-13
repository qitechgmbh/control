use std::time::Instant;

use control_core::controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController;
use serde::{Deserialize, Serialize};
use units::velocity::meter_per_second;
use units::{Acceleration, ConstZero, Jerk, Velocity};
use units::length::{meter, millimeter};
use units::f64::Length;

use crate::types::Direction;

use crate::speed_algorithms::{
    FixedSpeedAlgorithm,
    AdaptiveDiameterSpeedAlgorithm,
    DiameterData
};

#[derive(Debug)]
pub struct SpeedController
{
    enabled:    bool,
    direction:  Direction,
    speed:      Velocity,
    speed_mode: SpeedMode,
    algorithms: Algorithms,

    // currently all algorithms share acceleration controller
    // so we keep it in here. Might consider moving to each
    // algorithm and introduce a sort of sync function
    // for smooth transitions when switching algorithms
    acceleration_controller: LinearJerkSpeedController,
}

impl SpeedController
{
    pub fn new(
        fixed_speed_algorithm:    FixedSpeedAlgorithm,
        adaptive_speed_algorithm: AdaptiveDiameterSpeedAlgorithm,
        acceleration_controller:  LinearJerkSpeedController,
    ) -> Self
    {
        let fixed = FixedSpeedAlgorithm::new(speed_min, speed_max);

        let mut adaptive = AdaptiveDiameterSpeedAlgorithm::new();
        adaptive.set_speed_delta_max(0.33);
        adaptive.set_increase_per_step(0.033);
        adaptive.set_tolerance_limit(Length::new::<millimeter>(0.01));
        adaptive.set_adjustment_distance(Length::new::<meter>(0.5));

        let acceleration_controller = LinearJerkSpeedController::new_simple(
            Some(speed_max), 
            acceleration_max, 
            jerk_max
        );

        Self { 
            enabled:    false,
            direction:  Direction::Forward,
            speed_mode: SpeedMode::Fixed, 
            speed:      Velocity::ZERO,
            algorithms: Algorithms { fixed, diameter: adaptive },
            acceleration_controller, 
        }
    }

    pub fn update(&mut self, t: Instant)
    {
        use SpeedMode::*;

        let speed = match self.enabled 
        {
            true => match self.speed_mode 
            {
                Fixed => self.algorithms.fixed.compute(),
                DiameterAdaptive => self.algorithms.diameter,
            },
            false => Velocity::ZERO,    
        };

        let target_speed = speed * self.direction.multiplier();
        self.speed = self.acceleration_controller.update(target_speed, t);
    }

    pub fn speed_mode(&self) -> SpeedMode
    {
        self.speed_mode
    }

    pub fn set_speed_mode(&mut self, speed_mode: SpeedMode)
    {
        use SpeedMode::*;

        if self.speed_mode == speed_mode { return; }

        let desired_speed = match self.speed_mode 
        {
            Fixed            => self.algorithms.fixed.speed_target(),
            DiameterAdaptive => self.algorithms.diameter,
        };

        self.algorithm = algorithm;

        // take the base/current from current algorithm and transfer over for
        // smooth transitions
        match self.algorithm 
        {
            Fixed    => self.algorithms.fixed.set_speed_target(desired_speed),
            Adaptive => self.algorithms.diameter.set_speed_base(desired_speed),
        };
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
    pub diameter: AdaptiveDiameterSpeedAlgorithm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeedMode
{
    Fixed,
    DiameterAdaptive,
}

struct AlgorithmInput 
{
    diameter_data: Option<DiameterData>,
}