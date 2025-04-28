pub mod act;
pub mod api;
pub mod clamp_revolution;
pub mod linear_spool_speed_controller;
pub mod new;
pub mod spool_speed_controller;
pub mod tension_arm;

use std::{fmt::Debug, time::Instant};

use api::{
    ModeStateEvent, TensionArmAngleEvent, TensionArmStateEvent, TraverseStateEvent, Winder1Events,
    Winder1Namespace,
};
use chrono::DateTime;
use control_core::{
    actors::{
        digital_output_setter::DigitalOutputSetter, stepper_driver_el70x1::StepperDriverEL70x1,
    },
    converters::step_converter::StepConverter,
    machines::Machine,
    socketio::namespace::NamespaceCacheingLogic,
};
use spool_speed_controller::SpoolSpeedControllerTrait;
use tension_arm::TensionArm;
use uom::si::{angle::degree, angular_velocity::revolution_per_minute};

#[derive(Debug)]
pub struct Winder2 {
    // drivers
    // pub traverse_driver: StepperDriverPulseTrain,
    // pub puller_driver: StepperDriverPulseTrain,
    pub spool: StepperDriverEL70x1,
    pub tension_arm: TensionArm,
    pub laser: DigitalOutputSetter,

    // socketio
    namespace: Winder1Namespace,
    last_measurement_emit: DateTime<chrono::Utc>,

    // mode
    pub mode: Winder2Mode,

    // control circuit arm/spool
    pub spool_speed_controller: Box<dyn SpoolSpeedControllerTrait + Send + Sync>,
    pub spool_step_converter: StepConverter,
}

impl Machine for Winder2 {}

/// Implement Traverse
impl Winder2 {
    fn set_laser(&mut self, value: bool) {
        self.laser.set(value);
        self.emit_traverse_state();
    }

    fn emit_traverse_state(&mut self) {
        let event = TraverseStateEvent {
            laserpointer: self.laser.get(),
            ..Default::default()
        }
        .build();
        self.namespace
            .emit_cached(Winder1Events::TraverseState(event))
    }
}

/// Implement Mode
impl Winder2 {
    fn set_mode(&mut self, mode: &Winder2Mode) {
        // all transitions are allowed
        self.mode = mode.clone();

        // transiotion actions
        match mode {
            Winder2Mode::Standby => {
                // Spool
                self.spool.set_speed(0);
                self.spool.set_enabled(false);
                self.spool_speed_controller.set_enabled(false);
            }
            Winder2Mode::Hold => {
                // Spool
                self.spool.set_speed(0);
                self.spool.set_enabled(true);
                self.spool_speed_controller.set_enabled(false);
            }
            Winder2Mode::Pull => {
                // Spool
                self.spool.set_speed(0);
                self.spool.set_enabled(true);
                self.spool_speed_controller.set_enabled(false);
            }
            Winder2Mode::Wind => {
                // Spool
                self.spool.set_enabled(true);
                self.spool_speed_controller.reset();
                self.spool_speed_controller.set_enabled(true);
            }
        }
        self.emit_mode_state();
    }

    fn emit_mode_state(&mut self) {
        let event = ModeStateEvent {
            mode: self.mode.clone().into(),
        }
        .build();
        self.namespace.emit_cached(Winder1Events::Mode(event))
    }
}

/// Implement Tension Arm
impl Winder2 {
    fn tension_arm_zero(&mut self) {
        self.tension_arm.zero();
        self.emit_tension_arm_angle();
        self.emit_tension_arm_state();
    }

    /// called by `act`
    pub fn sync_spool_speed(&mut self, t: Instant) {
        let speed = self.spool_speed_controller.get_speed(t, &self.tension_arm);
        self.spool.set_speed(speed);
    }

    fn emit_tension_arm_angle(&mut self) {
        let event = TensionArmAngleEvent {
            degree: self.tension_arm.get_angle().get::<degree>(),
        }
        .build();
        self.namespace
            .emit_cached(Winder1Events::TensionArmAngleEvent(event))
    }

    fn emit_tension_arm_state(&mut self) {
        let event = TensionArmStateEvent {
            zeroed: self.tension_arm.zeroed,
        }
        .build();
        self.namespace
            .emit_cached(Winder1Events::TensionArmStateEvent(event))
    }
}

/// Implement Spool
impl Winder2 {
    pub fn spool_set_speed_max(&mut self, max_speed: f64) {
        self.spool_speed_controller.set_max_speed(max_speed);
        self.emit_spool_state();
    }

    pub fn spool_set_speed_min(&mut self, min_speed: f64) {
        self.spool_speed_controller.set_min_speed(min_speed);
        self.emit_spool_state();
    }

    fn emit_spool_rpm(&mut self) {
        let rpm = self
            .spool_step_converter
            .steps_to_angular_velocity(self.spool.get_speed() as f64)
            .get::<revolution_per_minute>();
        let event = api::SpoolRpmEvent { rpm }.build();
        self.namespace.emit_cached(Winder1Events::SpoolRpm(event))
    }

    fn emit_spool_state(&mut self) {
        let event = api::SpoolStateEvent {
            speed_min: self.spool_speed_controller.get_min_speed(),
            speed_max: self.spool_speed_controller.get_max_speed(),
        }
        .build();
        self.namespace.emit_cached(Winder1Events::SpoolState(event))
    }
}

#[derive(Debug, Clone, PartialEq)]
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
