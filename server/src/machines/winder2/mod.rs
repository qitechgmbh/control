pub mod act;
pub mod api;
pub mod clamp_revolution;
pub mod new;
pub mod puller_speed_controller;
pub mod spool_speed_controller;
pub mod tension_arm;
pub mod traverse_controller;

use std::{fmt::Debug, time::Instant};

use api::{
    ModeStateEvent, TensionArmAngleEvent, TensionArmStateEvent, TraversePositionEvent,
    TraverseStateEvent, Winder2Events, Winder2Namespace,
};
use control_core::{
    actors::{
        digital_input_getter::DigitalInputGetter, digital_output_setter::DigitalOutputSetter,
        stepper_driver_el70x1::StepperDriverEL70x1,
    },
    converters::angular_step_converter::AngularStepConverter,
    machines::Machine,
    socketio::namespace::NamespaceCacheingLogic,
    uom_extensions::velocity::meter_per_minute,
};
use puller_speed_controller::{PullerRegulationMode, PullerSpeedController};
use spool_speed_controller::SpoolSpeedController;
use tension_arm::TensionArm;
use traverse_controller::TraverseController;
use uom::si::{
    angle::degree,
    angular_velocity::revolution_per_minute,
    f64::{AngularVelocity, Length, Velocity},
    length::millimeter,
};

#[derive(Debug)]
pub struct Winder2 {
    // drivers
    pub traverse: StepperDriverEL70x1,
    pub puller: StepperDriverEL70x1,
    pub spool: StepperDriverEL70x1,
    pub tension_arm: TensionArm,
    pub laser: DigitalOutputSetter,

    // controllers
    pub traverse_controller: TraverseController,
    pub traverse_end_stop: DigitalInputGetter,

    // socketio
    namespace: Winder2Namespace,
    last_measurement_emit: Instant,

    // mode
    pub mode: Winder2Mode,
    pub spool_mode: SpoolMode,
    pub traverse_mode: TraverseMode,
    pub puller_mode: PullerMode,

    // control circuit arm/spool
    pub spool_speed_controller: SpoolSpeedController,
    pub spool_step_converter: AngularStepConverter,

    // control cirguit puller
    pub puller_speed_controller: PullerSpeedController,
}

impl Machine for Winder2 {}

/// Implement Traverse
impl Winder2 {
    fn set_laser(&mut self, value: bool) {
        self.laser.set(value);
        self.emit_traverse_state();
    }

    pub fn traverse_set_limit_inner(&mut self, limit: f64) {
        let limit = Length::new::<millimeter>(limit);
        self.traverse_controller.set_limit_inner(limit);
        self.emit_traverse_state();
    }

    pub fn traverse_set_limit_outer(&mut self, limit: f64) {
        let limit = Length::new::<millimeter>(limit);
        self.traverse_controller.set_limit_outer(limit);
        self.emit_traverse_state();
    }

    pub fn traverse_set_step_size(&mut self, step_size: f64) {
        let step_size = Length::new::<millimeter>(step_size);
        self.traverse_controller.set_step_size(step_size);
        self.emit_traverse_state();
    }

    pub fn traverse_set_padding(&mut self, padding: f64) {
        let padding = Length::new::<millimeter>(padding);
        self.traverse_controller.set_padding(padding);
        self.emit_traverse_state();
    }

    pub fn traverse_goto_limit_inner(&mut self) {
        // Only possible if homed, not standby, not traversing
        if !self.traverse_controller.is_homed()
            || self.traverse_mode == TraverseMode::Standby
            || self.traverse_controller.is_traversing()
        {
            return;
        }
        self.traverse_controller.goto_limit_inner();
        self.emit_traverse_state();
    }

    pub fn traverse_goto_limit_outer(&mut self) {
        // Only possible if homed, not standby, not traversing
        if !self.traverse_controller.is_homed()
            || self.traverse_mode == TraverseMode::Standby
            || self.traverse_controller.is_traversing()
        {
            return;
        }
        self.traverse_controller.goto_limit_outer();
        self.emit_traverse_state();
    }

    pub fn traverse_goto_home(&mut self) {
        // Only if not traversing
        if self.traverse_controller.is_traversing() {
            return;
        }
        self.traverse_controller.goto_home();
        self.emit_traverse_state();
    }

    pub fn emit_traverse_position(&mut self) {
        let position = self
            .traverse_controller
            .get_current_position()
            .map(|x| x.get::<millimeter>());
        let event = TraversePositionEvent { position }.build();
        self.namespace
            .emit_cached(Winder2Events::TraversePosition(event))
    }

    fn emit_traverse_state(&mut self) {
        let event = TraverseStateEvent {
            limit_inner: self
                .traverse_controller
                .get_limit_inner()
                .get::<millimeter>(),
            limit_outer: self
                .traverse_controller
                .get_limit_outer()
                .get::<millimeter>(),
            position_in: self
                .traverse_controller
                .get_limit_inner()
                .get::<millimeter>(),
            position_out: self
                .traverse_controller
                .get_limit_outer()
                .get::<millimeter>(),
            is_going_in: self.traverse_controller.is_going_in(),
            is_going_out: self.traverse_controller.is_going_out(),
            is_homed: self.traverse_controller.is_homed(),
            is_going_home: self.traverse_controller.is_going_home(),
            is_traversing: self.traverse_controller.is_traversing(),
            laserpointer: self.laser.get(),
            step_size: self.traverse_controller.get_step_size().get::<millimeter>(),
            padding: self.traverse_controller.get_padding().get::<millimeter>(),
        }
        .build();
        self.namespace
            .emit_cached(Winder2Events::TraverseState(event))
    }

    pub fn sync_traverse_speed(&mut self) {
        self.traverse_controller.update_speed(
            &mut self.traverse,
            &mut self.traverse_end_stop,
            self.spool_speed_controller.get_speed(),
        );
    }
}

/// Implement Mode
impl Winder2 {
    fn set_mode(&mut self, mode: &Winder2Mode) {
        // all transitions are allowed
        self.mode = mode.clone();

        // Apply the mode changes to the spool and puller
        self.set_spool_mode(mode);
        self.set_puller_mode(mode);
        self.set_traverse_mode(mode);

        self.emit_mode_state();
        self.emit_traverse_state();
    }

    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_spool_mode(&mut self, mode: &Winder2Mode) {
        // Convert to `Winder2Mode` to `SpoolMode`
        let mode: SpoolMode = mode.clone().into();

        // Transition matrix
        match self.spool_mode {
            SpoolMode::Standby => match mode {
                SpoolMode::Standby => {}
                SpoolMode::Hold => {
                    // From [`SpoolMode::Standby`] to [`SpoolMode::Hold`]
                    self.spool.set_enabled(true);
                }
                SpoolMode::Wind => {
                    // From [`SpoolMode::Standby`] to [`SpoolMode::Wind`]
                    self.spool.set_enabled(true);
                    self.spool_speed_controller.reset();
                    self.spool_speed_controller.set_enabled(true);
                }
            },
            SpoolMode::Hold => match mode {
                SpoolMode::Standby => {
                    // From [`SpoolMode::Hold`] to [`SpoolMode::Standby`]
                    self.spool.set_enabled(false);
                }
                SpoolMode::Hold => {}
                SpoolMode::Wind => {
                    // From [`SpoolMode::Hold`] to [`SpoolMode::Wind`]
                    self.spool_speed_controller.reset();
                    self.spool_speed_controller.set_enabled(true);
                }
            },
            SpoolMode::Wind => match mode {
                SpoolMode::Standby => {
                    // From [`SpoolMode::Wind`] to [`SpoolMode::Standby`]
                    self.spool.set_enabled(false);
                    self.spool_speed_controller.set_enabled(false);
                }
                SpoolMode::Hold => {
                    // From [`SpoolMode::Wind`] to [`SpoolMode::Hold`]
                    self.spool_speed_controller.set_enabled(false);
                }
                SpoolMode::Wind => {}
            },
        }

        // Update the internal state
        self.spool_mode = mode;
    }

    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_traverse_mode(&mut self, mode: &Winder2Mode) {
        // Convert to `Winder2Mode` to `TraverseMode`
        let mode: TraverseMode = mode.clone().into();

        // If coming out of standby
        if self.traverse_mode == TraverseMode::Standby && mode != TraverseMode::Standby {
            self.traverse.set_enabled(true);
            self.traverse_controller.set_enabled(true);
        }

        // If going into standby
        if mode == TraverseMode::Standby && self.traverse_mode != TraverseMode::Standby {
            // If we are going into standby, we need to stop the traverse
            self.traverse.set_enabled(false);
            self.traverse_controller.set_enabled(false);
        }

        // Transition matrix
        match self.traverse_mode {
            TraverseMode::Standby => match mode {
                TraverseMode::Standby => {}
                TraverseMode::Hold => {
                    // From [`TraverseMode::Standby`] to [`TraverseMode::Hold`]
                    self.traverse.set_enabled(true);
                    self.traverse_controller.set_enabled(true);
                    self.traverse_controller.goto_home();
                }
                TraverseMode::Traverse => {
                    // From [`TraverseMode::Standby`] to [`TraverseMode::Wind`]
                    self.traverse.set_enabled(true);
                    self.traverse_controller.set_enabled(true);
                    self.traverse_controller.start_traversing();
                }
            },
            TraverseMode::Hold => match mode {
                TraverseMode::Standby => {
                    // From [`TraverseMode::Hold`] to [`TraverseMode::Standby`]
                    self.traverse.set_enabled(false);
                    self.traverse_controller.set_enabled(false);
                }
                TraverseMode::Hold => {}
                TraverseMode::Traverse => {
                    // From [`TraverseMode::Hold`] to [`TraverseMode::Wind`]
                    self.traverse_controller.start_traversing();
                }
            },
            TraverseMode::Traverse => match mode {
                TraverseMode::Standby => {
                    // From [`TraverseMode::Wind`] to [`TraverseMode::Standby`]
                    self.traverse.set_enabled(false);
                    self.traverse_controller.set_enabled(false);
                }
                TraverseMode::Hold => {
                    // From [`TraverseMode::Wind`] to [`TraverseMode::Hold`]
                    self.traverse_controller.goto_home();
                }
                TraverseMode::Traverse => {}
            },
        }

        // Update the internal state
        self.traverse_mode = mode;
    }

    /// Apply the mode changes to the puller
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::puller_mode`]
    fn set_puller_mode(&mut self, mode: &Winder2Mode) {
        // Convert to `Winder2Mode` to `PullerMode`
        let mode: PullerMode = mode.clone().into();

        // Transition matrix
        match self.puller_mode {
            PullerMode::Standby => match mode {
                PullerMode::Standby => {}
                PullerMode::Hold => {
                    // From [`PullerMode::Standby`] to [`PullerMode::Hold`]
                    self.puller.set_enabled(true);
                }
                PullerMode::Pull => {
                    // From [`PullerMode::Standby`] to [`PullerMode::Pull`]
                    self.puller.set_enabled(true);
                    self.puller_speed_controller.set_enabled(true);
                }
            },
            PullerMode::Hold => match mode {
                PullerMode::Standby => {
                    // From [`PullerMode::Hold`] to [`PullerMode::Standby`]
                    self.puller.set_enabled(false);
                }
                PullerMode::Hold => {}
                PullerMode::Pull => {
                    // From [`PullerMode::Hold`] to [`PullerMode::Pull`]
                    self.puller_speed_controller.set_enabled(true);
                }
            },
            PullerMode::Pull => match mode {
                PullerMode::Standby => {
                    // From [`PullerMode::Pull`] to [`PullerMode::Standby`]
                    self.puller.set_enabled(false);
                    self.puller_speed_controller.set_enabled(false);
                }
                PullerMode::Hold => {
                    // From [`PullerMode::Pull`] to [`PullerMode::Hold`]
                    self.puller_speed_controller.set_enabled(false);
                }
                PullerMode::Pull => {}
            },
        }

        // Update the internal state
        self.puller_mode = mode;
    }

    fn emit_mode_state(&mut self) {
        let event = ModeStateEvent {
            mode: self.mode.clone().into(),
        }
        .build();
        self.namespace.emit_cached(Winder2Events::Mode(event))
    }
}

/// Implement Tension Arm
impl Winder2 {
    fn tension_arm_zero(&mut self) {
        self.tension_arm.zero();
        self.emit_tension_arm_angle();
        self.emit_tension_arm_state();
    }

    fn emit_tension_arm_angle(&mut self) {
        let event = TensionArmAngleEvent {
            degree: self.tension_arm.get_angle().get::<degree>(),
        }
        .build();
        self.namespace
            .emit_cached(Winder2Events::TensionArmAngleEvent(event))
    }

    fn emit_tension_arm_state(&mut self) {
        let event = TensionArmStateEvent {
            zeroed: self.tension_arm.zeroed,
        }
        .build();
        self.namespace
            .emit_cached(Winder2Events::TensionArmStateEvent(event))
    }
}

/// Implement Spool
impl Winder2 {
    /// called by `act`
    pub fn sync_spool_speed(&mut self, t: Instant) {
        let angular_velocity = self
            .spool_speed_controller
            .get_angular_velocity(t, &self.tension_arm);
        let steps_per_second = self
            .spool_step_converter
            .angular_velocity_to_steps(angular_velocity);
        self.spool.set_speed(steps_per_second as i32);
    }

    pub fn spool_set_speed_max(&mut self, max_speed: f64) {
        let max_speed = AngularVelocity::new::<revolution_per_minute>(max_speed);
        self.spool_speed_controller.set_max_speed(max_speed);
        self.emit_spool_state();
    }

    pub fn spool_set_speed_min(&mut self, min_speed: f64) {
        let min_speed = AngularVelocity::new::<revolution_per_minute>(min_speed);
        self.spool_speed_controller.set_min_speed(min_speed);
        self.emit_spool_state();
    }

    fn emit_spool_rpm(&mut self) {
        let rpm = self
            .spool_step_converter
            .steps_to_angular_velocity(self.spool.get_speed() as f64)
            .get::<revolution_per_minute>();
        let event = api::SpoolRpmEvent { rpm }.build();
        self.namespace.emit_cached(Winder2Events::SpoolRpm(event))
    }

    fn emit_spool_state(&mut self) {
        // Convert angular velocity to steps/second
        let speed_min = self
            .spool_speed_controller
            .get_min_speed()
            .get::<revolution_per_minute>();
        // Convert angular velocity to steps/second
        let speed_max = self
            .spool_speed_controller
            .get_max_speed()
            .get::<revolution_per_minute>();
        let event = api::SpoolStateEvent {
            speed_min,
            speed_max,
        }
        .build();
        self.namespace.emit_cached(Winder2Events::SpoolState(event))
    }
}

/// Implement Puller
impl Winder2 {
    /// called by `act`
    pub fn sync_puller_speed(&mut self, t: Instant) {
        let angular_velocity = self.puller_speed_controller.get_angular_velocity(t);
        let steps_per_second = self
            .puller_speed_controller
            .converter
            .angular_velocity_to_steps(angular_velocity);
        self.puller.set_speed(steps_per_second as i32);
    }

    pub fn puller_set_regulation(&mut self, puller_regulation_mode: PullerRegulationMode) {
        self.puller_speed_controller
            .set_regulation_mode(puller_regulation_mode);
        self.emit_puller_state();
    }

    /// Set target speed in m/min
    pub fn puller_set_target_speed(&mut self, target_speed: f64) {
        // Convert m/min to velocity
        let target_speed = Velocity::new::<meter_per_minute>(target_speed);
        self.puller_speed_controller.set_target_speed(target_speed);
        self.emit_puller_state();
    }

    /// Set target diameter in mm
    pub fn puller_set_target_diameter(&mut self, target_diameter: f64) {
        // Convert m/min to velocity
        let target_diameter = Length::new::<millimeter>(target_diameter);
        self.puller_speed_controller
            .set_target_diameter(target_diameter);
        self.emit_puller_state();
    }

    pub fn emit_puller_speed(&mut self) {
        let steps_per_second = self.puller.get_speed();
        let angular_velocity = self
            .puller_speed_controller
            .converter
            .steps_to_angular_velocity(steps_per_second as f64);
        let speed = self
            .puller_speed_controller
            .angular_velocity_to_speed(angular_velocity);
        let event = api::Winder2Events::PullerSpeed(
            api::PullerSpeedEvent {
                speed: speed.get::<meter_per_minute>(),
            }
            .build(),
        );
        self.namespace.emit_cached(event);
    }

    pub fn emit_puller_state(&mut self) {
        let event = api::Winder2Events::PullerState(
            api::PullerStateEvent {
                regulation: self.puller_speed_controller.regulation_mode.clone(),
                target_speed: self
                    .puller_speed_controller
                    .target_speed
                    .get::<meter_per_minute>(),
                target_diameter: self
                    .puller_speed_controller
                    .target_diameter
                    .get::<millimeter>(),
            }
            .build(),
        );
        self.namespace.emit_cached(event);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Winder2Mode {
    Standby,
    Hold,
    Pull,
    Wind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpoolMode {
    Standby,
    Hold,
    Wind,
}

impl From<Winder2Mode> for SpoolMode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => SpoolMode::Standby,
            Winder2Mode::Hold => SpoolMode::Hold,
            Winder2Mode::Pull => SpoolMode::Hold,
            Winder2Mode::Wind => SpoolMode::Wind,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraverseMode {
    Standby,
    Hold,
    Traverse,
}

impl From<Winder2Mode> for TraverseMode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => TraverseMode::Standby,
            Winder2Mode::Hold => TraverseMode::Hold,
            Winder2Mode::Pull => TraverseMode::Hold,
            Winder2Mode::Wind => TraverseMode::Traverse,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PullerMode {
    Standby,
    Hold,
    Pull,
}

impl From<Winder2Mode> for PullerMode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => PullerMode::Standby,
            Winder2Mode::Hold => PullerMode::Hold,
            Winder2Mode::Pull => PullerMode::Pull,
            Winder2Mode::Wind => PullerMode::Pull,
        }
    }
}

impl std::fmt::Display for Winder2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Winder2")
    }
}
