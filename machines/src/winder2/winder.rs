#[cfg(not(feature = "mock-machine"))]
use super::{
    base::MachineBase, 
    types::Mode,
    devices::Puller,
    devices::TensionArm,
};

#[cfg(not(feature = "mock-machine"))]
#[derive(Debug)]
pub struct Winder2
{
    base: MachineBase,

    // machine_config:
    mode: Mode,

    // devices
    // --------------------------
    // traverse:    Traverse,
    puller: Puller,
    // spool:       Spool,
    tension_arm: TensionArm
    // laser:       Laser
}

impl Winder2
{
    // pub const fn can_wind(&self) -> bool 
    // {
    //     // Check if tension arm is zeroed and traverse is homed
    //     self.tension_arm.zeroed
    //         && self.traverse.is_homed()
    //         && !self.traverse.is_going_home()
    // }
}