use qitech_framework::EnumProperty;

use crate::machines::winder_v2::Mode as WinderMode;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumProperty)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
}

impl From<WinderMode> for Mode {
    fn from(mode: WinderMode) -> Self {
        match mode {
            WinderMode::Standby => Self::Standby,
            WinderMode::Hold => Self::Hold,
            WinderMode::Pull => Self::Pull,
            WinderMode::Wind => Self::Pull,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, EnumProperty)]
#[allow(clippy::enum_variant_names)]
pub enum GearRatio {
    #[default]
    OneToOne,
    OneToFive,
    OneToTen,
}

impl GearRatio {
    /// Get the speed multiplier for this gear ratio
    pub fn multiplier(&self) -> f64 {
        match self {
            GearRatio::OneToOne => 1.0,
            GearRatio::OneToFive => 5.0,
            GearRatio::OneToTen => 10.0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, EnumProperty)]
pub enum SpeedControlAlgorithm {
    #[default]
    Direct,
    Adaptive,
}
