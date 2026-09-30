use qitech_framework::EnumProperty;

use crate::machines::winder_v2::Mode as WinderMode;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumProperty)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Wind,
}

impl From<WinderMode> for Mode {
    fn from(mode: WinderMode) -> Self {
        match mode {
            WinderMode::Standby => Self::Standby,
            WinderMode::Hold => Self::Hold,
            WinderMode::Pull => Self::Hold,
            WinderMode::Wind => Self::Wind,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, EnumProperty)]
pub enum SpeedControlAlgorithm {
    #[default]
    Adaptive,
    MinMax,
}
