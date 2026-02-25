use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;

mod types;

pub use types::GearRatio;
pub use types::RegulationMode;

pub struct Puller
{
    enabled:   bool,
    device:    StepperVelocityEL70x1,
    regulator: SpeedRegulator,
    forward:   bool,
    ratio:     GearRatio,

    // converter:  LinearStepConverter
    // last_speed: Velocity
}

pub enum SpeedRegulator
{
    Manual,
    Diameter,
}