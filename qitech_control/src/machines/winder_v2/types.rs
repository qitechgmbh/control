use qitech_framework::{EnumProperty, MachineIdentificationUnique, machine::RemoteProperty};
use qitech_lib::units::Length;

#[derive(Debug, Clone, Copy, Default, PartialEq, EnumProperty)]
pub enum SpoolAutomaticActionMode {
    #[default]
    NoAction,
    Pull,
    Hold,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, EnumProperty)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpoolMode {
    Standby,
    Hold,
    Wind,
}

impl From<Mode> for SpoolMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Standby => Self::Standby,
            Mode::Hold => Self::Hold,
            Mode::Pull => Self::Hold,
            Mode::Wind => Self::Wind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraverseMode {
    Standby,
    Hold,
    Traverse,
}

impl From<Mode> for TraverseMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Standby => Self::Standby,
            Mode::Hold => Self::Hold,
            Mode::Pull => Self::Hold,
            Mode::Wind => Self::Traverse,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullerMode {
    Standby,
    Hold,
    Pull,
}

impl From<Mode> for PullerMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Standby => Self::Standby,
            Mode::Hold => Self::Hold,
            Mode::Pull => Self::Pull,
            Mode::Wind => Self::Pull,
        }
    }
}

pub struct LaserSubscription {
    pub ident: MachineIdentificationUnique,
    pub current: RemoteProperty<Length>,
    pub target: RemoteProperty<Length>,
    pub lower: RemoteProperty<Length>,
    pub upper: RemoteProperty<Length>,
}
