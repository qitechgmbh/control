use qitech_framework::{EnumProperty, MachineIdentificationUnique, machine::SubscribedProperty};
use qitech_lib::units::Length;

#[derive(Debug, Clone, Copy, Default, PartialEq, EnumProperty)]
pub enum SpoolAutomaticActionMode {
    #[default]
    NoAction,
    Pull,
    Hold,
}

#[derive(Debug, Clone, Default, PartialEq, EnumProperty)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}

impl From<Winder2Mode> for Mode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Self::Standby,
            Winder2Mode::Hold => Self::Hold,
            Winder2Mode::Pull => Self::Pull,
            Winder2Mode::Wind => Self::Wind,
        }
    }
}

impl From<Mode> for Winder2Mode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Standby => Self::Standby,
            Mode::Hold => Self::Hold,
            Mode::Pull => Self::Pull,
            Mode::Wind => Self::Wind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Winder2Mode {
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

impl From<Winder2Mode> for SpoolMode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Self::Standby,
            Winder2Mode::Hold => Self::Hold,
            Winder2Mode::Pull => Self::Hold,
            Winder2Mode::Wind => Self::Wind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraverseMode {
    Standby,
    Hold,
    Traverse,
}

impl From<Winder2Mode> for TraverseMode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Self::Standby,
            Winder2Mode::Hold => Self::Hold,
            Winder2Mode::Pull => Self::Hold,
            Winder2Mode::Wind => Self::Traverse,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullerMode {
    Standby,
    Hold,
    Pull,
}

impl From<Winder2Mode> for PullerMode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Self::Standby,
            Winder2Mode::Hold => Self::Hold,
            Winder2Mode::Pull => Self::Pull,
            Winder2Mode::Wind => Self::Pull,
        }
    }
}

pub struct LaserSubscription {
    pub ident: MachineIdentificationUnique,
    pub current: SubscribedProperty<Length>,
    pub target: SubscribedProperty<Length>,
    pub lower: SubscribedProperty<Length>,
    pub upper: SubscribedProperty<Length>,
}
