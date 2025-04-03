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
        digital_output_setter::DigitalOutputSetter, stepper_driver_el7031::StepperDriverEl7031,
    },
    machines::Machine,
    socketio::{event::EventBuilder, room::RoomCacheingLogic},
};
use tension_arm::TensionArm;
use uom::si::angle::degree;

#[derive(Debug)]
pub struct WinderV1 {
    // drivers
    // pub traverse_driver: StepperDriverPulseTrain,
    // pub puller_driver: StepperDriverPulseTrain,
    pub winder: StepperDriverEl7031,
    pub tension_arm: TensionArm,
    pub laser: DigitalOutputSetter,

    // socketio
    room: Winder1Room,
    last_measurement_emit: DateTime<chrono::Utc>,

    // mode
    pub mode: WinderV1Mode,
}

impl Machine for WinderV1 {}

impl WinderV1 {
    pub fn set_mode(&mut self, mode: &WinderV1Mode) {
        // all transitions are allowed
        self.mode = mode.clone();

        // transiotion actions
        match mode {
            WinderV1Mode::Standby => {
                // self.traverse_driver.set_frequency(0);
                // self.puller_driver.set_frequency(0);
                self.winder.set_speed(0);
                self.winder.set_enabled(false);
            }
            WinderV1Mode::Hold => {
                // self.puller_driver.set_frequency(0);
                // self.winder_driver.set_frequency(0);
            }
            WinderV1Mode::Pull => {
                // self.traverse_driver.set_frequency(10);
                // self.puller_driver.set_frequency(200);
            }
            WinderV1Mode::Wind => {
                self.winder.set_enabled(true);
                self.winder.set_speed(200);
                // self.winder_driver.set_frequency(20000);
                // self.puller_driver.set_frequency(1000);
            }
        }
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
        self.laser.set(value);
        self.emit_traverse_state();
    }

    pub fn emit_traverse_state(&mut self) {
        let event = TraverseStateEvent {
            laserpointer: self.laser.get(),
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
