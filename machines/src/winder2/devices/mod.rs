mod spool;
pub use spool::Spool;
pub use spool::SpeedControlMode as SpoolSpeedControlMode;

mod puller;
pub use puller::Puller;
pub use puller::GearRatio as PullerGearRatio;
pub use puller::SpeedControlMode as PullerSpeedControlMode;

mod tension_arm;
pub use tension_arm::TensionArm;

mod traverse;
pub use traverse::Traverse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationState
{
    Disabled,
    Holding,
    Running,
}