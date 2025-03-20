pub mod act;
pub mod api;
pub mod new;

use super::Machine;
use crate::socketio::event::EventBuilder;
use crate::socketio::room::room::RoomCacheingLogic;
use api::{TraverseStateEvent, Winder1Events, Winder1Room};
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

    // socketio
    room: Winder1Room,
}

impl Machine for WinderV1 {}

impl WinderV1 {
    pub fn set_laser(&mut self, value: bool) {
        self.laser_driver.set(value);
        self.emit_traverse_state();
    }

    pub fn emit_traverse_state(&mut self) {
        let event = TraverseStateEvent {
            laserpointer: self.laser_driver.get(),
            ..Default::default()
        }
        .build();
        self.room.emit_cached(Winder1Events::TraverseState(event));
    }
}

impl std::fmt::Display for WinderV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WinderV1")
    }
}
