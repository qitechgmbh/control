use super::{ AdaptiveSpeedController, MinMaxSpeedController };

#[derive(Debug)]
pub struct SpeedControllers
{
    pub minmax:   MinMaxSpeedController,
    pub adaptive: AdaptiveSpeedController,
}

impl SpeedControllers
{
    pub fn new() -> Self
    {
        Self { 
            minmax: MinMaxSpeedController::new(), 
            adaptive: AdaptiveSpeedController::new() 
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedControlMode
{
    Adaptive,
    MinMax,
}