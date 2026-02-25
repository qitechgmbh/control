use serde::{Deserialize, Serialize};

// regulation mode
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum RegulationMode 
{
    #[default]
    Speed,
    Diameter,
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

impl GearRatio {

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