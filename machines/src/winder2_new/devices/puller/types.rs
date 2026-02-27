use serde::{Deserialize, Serialize};

pub use crate::types::Direction;

// state
#[derive(Debug,Clone, Copy, PartialEq, Eq)]
pub enum State 
{
    Disabled,
    Holding,
    Running,
}

// gear ratio
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[derive(Default)]
pub enum GearRatio 
{
    #[default]
    OneToOne,
    OneToFive,
    OneToTen,
}

impl GearRatio 
{
    /// Get the speed multiplier for this gear ratio
    pub fn multiplier(&self) -> f64 
    {
        match self 
        {
            GearRatio::OneToOne => 1.0,
            GearRatio::OneToFive => 5.0,
            GearRatio::OneToTen => 10.0,
        }
    }
}