pub mod act;
pub mod api;
pub mod new;

use super::Machine;
use ethercat_hal::actors::analog_input_logger::AnalogInputLogger;
use ethercat_hal::actors::digital_output_setter::DigitalOutputSetter;
use ethercat_hal::actors::stepper_driver_pulse_train::StepperDriverPulseTrain;

#[derive(Debug)]
pub struct WinderV1 {
    // drivers
    pub traverse_driver: StepperDriverPulseTrain,
    pub puller_driver: StepperDriverPulseTrain,
    pub winder_driver: StepperDriverPulseTrain,
    pub tension_arm_driver: AnalogInputLogger,
    pub laser_driver: DigitalOutputSetter,
}

impl Machine for WinderV1 {}

impl std::fmt::Display for WinderV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WinderV1")
    }
}
