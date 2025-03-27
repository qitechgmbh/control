pub mod act;
pub mod api;
pub mod new;
pub mod tension_arm;

use api::{
    MeasurementsTensionArmEvent, ModeStateEvent, TraverseStateEvent, Winder1Events, Winder1Room,
};
use chrono::DateTime;
use control_core::{
    actors::{
        digital_output_setter::DigitalOutputSetter,
        stepper_driver_pulse_train::StepperDriverPulseTrain,
    },
    machines::Machine,
    socketio::{event::EventBuilder, room::RoomCacheingLogic},
};
use tension_arm::TensionArm;
use uom::si::angle::degree;

#[derive(Debug)]
pub struct WinderV1 {
    // drivers
    pub traverse_driver: StepperDriverPulseTrain,
    pub puller_driver: StepperDriverPulseTrain,
    pub winder_driver: StepperDriverPulseTrain,
    pub tension_arm: TensionArm,
    pub laser_driver: DigitalOutputSetter,

    // socketio
    room: Winder1Room,
    last_measurement_emit: DateTime<chrono::Utc>,

    // mode
    pub mode: WinderV1Mode,
}

impl Machine for WinderV1 {}

impl WinderV1 {
    pub fn set_mode(&mut self, state: &WinderV1Mode) {
        // all transitions are allowed
        self.mode = state.clone();
        self.emit_mode_state();
    }

    pub fn emit_mode_state(&mut self) {
        let event = ModeStateEvent {
            mode: self.mode.clone().into(),
        }
        .build();
        self.room.emit_cached(Winder1Events::Mode(event));
    }

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

    pub fn emit_measurement_tension_arm(&mut self) {
        let event = MeasurementsTensionArmEvent {
            degree: self.tension_arm.get_angle().get::<degree>(),
        }
        .build();
        self.room
            .emit_cached(Winder1Events::MeasurementsTensionArm(event));
    }
}

#[derive(Debug, Clone)]
pub enum WinderV1Mode {
    Standby,
    Hold,
    Pull,
    Wind,
}

impl std::fmt::Display for WinderV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WinderV1")
    }
}
