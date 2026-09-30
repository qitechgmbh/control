use qitech_framework::EnumProperty;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::machine::RemoteProperty;
use qitech_lib::units::Length;

#[derive(Debug, Clone, Copy, Default, PartialEq, EnumProperty)]
pub enum AutomaticActionSpoolAction {
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

pub struct LaserSubscription {
    pub ident: MachineInstanceIdentification,
    pub diameter: RemoteProperty<Length>,
    pub diameter_target: RemoteProperty<Length>,
}
