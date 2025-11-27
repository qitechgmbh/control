// Controller modules for Gluetex machine

pub mod adaptive_spool_speed_controller;
pub mod addon_motor_controller;
pub mod minmax_spool_speed_controller;
pub mod puller_speed_controller;
pub mod slave_puller_speed_controller;
pub mod spool_speed_controller;
pub mod temperature_controller;
pub mod tension_arm;
pub mod traverse_controller;

// Re-export main controller types
pub use adaptive_spool_speed_controller::*;
pub use addon_motor_controller::AddonMotorController;
pub use minmax_spool_speed_controller::*;
pub use puller_speed_controller::PullerSpeedController;
pub use slave_puller_speed_controller::SlavePullerSpeedController;
pub use spool_speed_controller::SpoolSpeedController;
pub use temperature_controller::TemperatureController;
pub use tension_arm::TensionArm;
pub use traverse_controller::TraverseController;
