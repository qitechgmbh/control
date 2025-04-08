pub mod act;
pub mod api;
pub mod new;
pub mod tension_arm;

use api::{
    MeasurementsTensionArmEvent, ModeStateEvent, TraverseStateEvent, Winder1Events,
    Winder1Namespace,
};
use chrono::DateTime;
use control_core::{
    actors::{
        digital_output_setter::DigitalOutputSetter, stepper_driver_el70x1::StepperDriverEL70x1,
    },
    machines::Machine,
    socketio::{event::EventBuilder, namespace::NamespaceCacheingLogic},
};
use tension_arm::TensionArm;
use uom::si::angle::degree;

#[derive(Debug)]
pub struct Winder2 {
    // drivers
    // pub traverse_driver: StepperDriverPulseTrain,
    // pub puller_driver: StepperDriverPulseTrain,
    pub winder: StepperDriverEL70x1,
    pub tension_arm: TensionArm,
    pub laser: DigitalOutputSetter,

    // socketio
    namespace: Winder1Namespace,
    last_measurement_emit: DateTime<chrono::Utc>,

    // mode
    pub mode: Winder2Mode,
}

impl Machine for Winder2 {}

impl Winder2 {
    pub fn set_mode(&mut self, mode: &Winder2Mode) {
        // all transitions are allowed
        self.mode = mode.clone();

        // transiotion actions
        match mode {
            Winder2Mode::Standby => {
                self.winder.set_speed(0);
                self.winder.set_enabled(false);
            }
            Winder2Mode::Hold => {}
            Winder2Mode::Pull => {}
            Winder2Mode::Wind => {
                self.winder.set_enabled(true);
                self.winder.set_speed(1000);
            }
        }
        self.emit_mode_state();
    }

    pub fn emit_mode_state(&mut self) {
        let event = ModeStateEvent {
            mode: self.mode.clone().into(),
        }
        .build();
        self.namespace.emit_cached(Winder1Events::Mode(event));
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
        self.namespace
            .emit_cached(Winder1Events::TraverseState(event));
    }

    pub fn emit_measurement_tension_arm(&mut self) {
        let event = MeasurementsTensionArmEvent {
            degree: self.tension_arm.get_angle().get::<degree>(),
        }
        .build();
        self.namespace
            .emit_cached(Winder1Events::MeasurementsTensionArm(event));
    }
}

#[derive(Debug, Clone)]
pub enum Winder2Mode {
    Standby,
    Hold,
    Pull,
    Wind,
}

impl std::fmt::Display for Winder2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Winder2")
    }
}
