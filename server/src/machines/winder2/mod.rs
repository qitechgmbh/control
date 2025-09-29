pub mod act;
pub mod adaptive_spool_speed_controller;
pub mod api;
pub mod clamp_revolution;
pub mod emit;
pub mod filament_tension;
pub mod minmax_spool_speed_controller;
pub mod new;
pub mod puller_speed_controller;
pub mod spool_speed_controller;
pub mod tension_arm;
pub mod traverse_controller;

use api::{SpoolAutomaticActionMode, Winder2Namespace};
use control_core::{
    converters::angular_step_converter::AngularStepConverter,
    machines::{
        ConnectedMachine,
        identification::{MachineIdentification, MachineIdentificationUnique},
        manager::MachineManager,
    },
};

use control_core_derive::Machine;
use ethercat_hal::io::{
    digital_input::DigitalInput, digital_output::DigitalOutput,
    stepper_velocity_el70x1::StepperVelocityEL70x1,
};
use puller_speed_controller::PullerSpeedController;
use smol::lock::{Mutex, RwLock};
use spool_speed_controller::SpoolSpeedController;
use std::{fmt::Debug, sync::Weak, time::Instant};
use tension_arm::TensionArm;
use traverse_controller::TraverseController;
use uom::{
    ConstZero,
    si::{
        f64::Length,
        length::{meter, millimeter},
        velocity::meter_per_second,
    },
};

use crate::machines::{MACHINE_WINDER_V1, VENDOR_QITECH, buffer1::BufferV1};

#[derive(Debug)]
pub struct SpoolAutomaticAction {
    pub progress: Length,
    progress_last_check: Instant,
    pub target_length: Length,
    pub mode: SpoolAutomaticActionMode,
}

#[derive(Debug, Machine)]
pub struct Winder2 {
    // drivers
    pub traverse: StepperVelocityEL70x1,
    pub puller: StepperVelocityEL70x1,
    pub spool: StepperVelocityEL70x1,
    pub tension_arm: TensionArm,
    pub laser: DigitalOutput,

    // controllers
    pub traverse_controller: TraverseController,
    pub traverse_end_stop: DigitalInput,

    // socketio
    namespace: Winder2Namespace,
    last_measurement_emit: Instant,

    // machine connection
    pub machine_manager: Weak<RwLock<MachineManager>>,
    pub machine_identification_unique: MachineIdentificationUnique,

    // connected machines
    pub connected_buffer: Option<ConnectedMachine<Weak<Mutex<BufferV1>>>>,

    // mode
    pub mode: Winder2Mode,
    pub spool_mode: SpoolMode,
    pub traverse_mode: TraverseMode,
    pub puller_mode: PullerMode,

    // control circuit arm/spool
    pub spool_speed_controller: SpoolSpeedController,
    pub spool_step_converter: AngularStepConverter,

    // spool automatic action state
    pub spool_automatic_action: SpoolAutomaticAction,

    // control circuit puller
    pub puller_speed_controller: PullerSpeedController,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
}

impl Winder2 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_WINDER_V1,
    };
}

/// Implement Traverse helpers
impl Winder2 {
    fn set_laser(&mut self, value: bool) {
        self.laser.set(value);
    }

    fn validate_traverse_limits(inner: Length, outer: Length) -> bool {
        outer > inner + Length::new::<millimeter>(0.9)
    }

    pub fn sync_traverse_speed(&mut self) {
        self.traverse_controller.update_speed(
            &mut self.traverse,
            &self.traverse_end_stop,
            self.spool_speed_controller.get_speed(),
        )
    }

    pub const fn can_wind(&self) -> bool {
        self.tension_arm.zeroed
            && self.traverse_controller.is_homed()
            && !self.traverse_controller.is_going_home()
    }

    pub fn can_go_in(&self) -> bool {
        self.traverse_controller.is_homed()
            && self.traverse_mode != TraverseMode::Standby
            && !self.traverse_controller.is_going_in()
            && !self.traverse_controller.is_going_home()
            && !self.traverse_controller.is_traversing()
            && self.mode != Winder2Mode::Wind
    }

    pub fn can_go_out(&self) -> bool {
        self.traverse_controller.is_homed()
            && self.traverse_mode != TraverseMode::Standby
            && !self.traverse_controller.is_going_out()
            && !self.traverse_controller.is_going_home()
            && !self.traverse_controller.is_traversing()
            && self.mode != Winder2Mode::Wind
    }

    pub fn can_go_home(&self) -> bool {
        self.traverse_mode != TraverseMode::Standby
            && !self.traverse_controller.is_going_home()
            && !self.traverse_controller.is_traversing()
            && self.mode != Winder2Mode::Wind
    }
}

/// Implement Mode transitions (without emit)
impl Winder2 {
    fn set_spool_mode(&mut self, mode: &Winder2Mode) {
        let mode: SpoolMode = mode.clone().into();
        match self.spool_mode {
            SpoolMode::Standby => match mode {
                SpoolMode::Hold => self.spool.set_enabled(true),
                SpoolMode::Wind => {
                    self.spool.set_enabled(true);
                    self.spool_speed_controller.set_enabled(true);
                }
                _ => {}
            },
            SpoolMode::Hold => match mode {
                SpoolMode::Standby => self.spool.set_enabled(false),
                SpoolMode::Wind => self.spool_speed_controller.set_enabled(true),
                _ => {}
            },
            SpoolMode::Wind => match mode {
                SpoolMode::Standby => {
                    self.spool.set_enabled(false);
                    self.spool_speed_controller.set_enabled(false);
                }
                SpoolMode::Hold => self.spool_speed_controller.set_enabled(false),
                _ => {}
            },
        }
        self.spool_mode = mode;
    }

    fn set_traverse_mode(&mut self, mode: &Winder2Mode) {
        let mode: TraverseMode = mode.clone().into();

        if self.traverse_mode == TraverseMode::Standby && mode != TraverseMode::Standby {
            self.traverse.set_enabled(true);
            self.traverse_controller.set_enabled(true);
        }
        if mode == TraverseMode::Standby && self.traverse_mode != TraverseMode::Standby {
            self.traverse.set_enabled(false);
            self.traverse_controller.set_enabled(false);
        }

        match self.traverse_mode {
            TraverseMode::Standby => match mode {
                TraverseMode::Hold => self.traverse_controller.goto_home(),
                TraverseMode::Traverse => self.traverse_controller.start_traversing(),
                _ => {}
            },
            TraverseMode::Hold => match mode {
                TraverseMode::Traverse => self.traverse_controller.start_traversing(),
                _ => {}
            },
            TraverseMode::Traverse => match mode {
                TraverseMode::Hold => self.traverse_controller.goto_home(),
                _ => {}
            },
        }

        self.traverse_mode = mode;
    }

    fn set_puller_mode(&mut self, mode: &Winder2Mode) {
        let mode: PullerMode = mode.clone().into();
        match self.puller_mode {
            PullerMode::Standby => match mode {
                PullerMode::Hold => self.puller.set_enabled(true),
                PullerMode::Pull => {
                    self.puller.set_enabled(true);
                    self.puller_speed_controller.set_enabled(true);
                }
                _ => {}
            },
            PullerMode::Hold => match mode {
                PullerMode::Standby => self.puller.set_enabled(false),
                PullerMode::Pull => self.puller_speed_controller.set_enabled(true),
                _ => {}
            },
            PullerMode::Pull => match mode {
                PullerMode::Standby => {
                    self.puller.set_enabled(false);
                    self.puller_speed_controller.set_enabled(false);
                }
                PullerMode::Hold => self.puller_speed_controller.set_enabled(false),
                _ => {}
            },
        }
        self.puller_mode = mode;
    }
}

/// Implement Spool calculations
impl Winder2 {
    pub fn sync_spool_speed(&mut self, t: Instant) {
        let angular_velocity = self.spool_speed_controller.update_speed(
            t,
            &self.tension_arm,
            &self.puller_speed_controller,
        );
        let steps_per_second = self
            .spool_step_converter
            .angular_velocity_to_steps(angular_velocity);
        let _ = self.spool.set_speed(steps_per_second);
    }

    pub fn stop_or_pull_spool(&mut self, now: Instant) {
        if matches!(
            self.spool_automatic_action.mode,
            SpoolAutomaticActionMode::NoAction
        ) {
            self.calculate_spool_auto_progress_(now);
            return;
        }

        match self.mode {
            Winder2Mode::Pull | Winder2Mode::Wind => self.calculate_spool_auto_progress_(now),
            _ => {
                self.spool_automatic_action.progress_last_check = now;
                return;
            }
        }

        if self.spool_automatic_action.progress >= self.spool_automatic_action.target_length {
            match self.spool_automatic_action.mode {
                SpoolAutomaticActionMode::Pull => {
                    self.stop_or_pull_spool_reset(now);
                    self.mode = Winder2Mode::Pull;
                }
                SpoolAutomaticActionMode::Hold => {
                    self.stop_or_pull_spool_reset(now);
                    self.mode = Winder2Mode::Hold;
                }
                _ => {}
            }
        }
    }

    pub const fn stop_or_pull_spool_reset(&mut self, now: Instant) {
        self.spool_automatic_action.progress = Length::ZERO;
        self.spool_automatic_action.progress_last_check = now;
    }

    pub fn calculate_spool_auto_progress_(&mut self, now: Instant) {
        let dt = now
            .duration_since(self.spool_automatic_action.progress_last_check)
            .as_secs_f64();

        let meters_pulled_this_interval = Length::new::<meter>(
            self.puller_speed_controller
                .last_speed
                .get::<meter_per_second>()
                * dt,
        );

        self.spool_automatic_action.progress += meters_pulled_this_interval;
        self.spool_automatic_action.progress_last_check = now;
    }
}

/// Implement Puller calculations
impl Winder2 {
    pub fn sync_puller_speed(&mut self, t: Instant) {
        let angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
        let steps_per_second = self
            .puller_speed_controller
            .converter
            .angular_velocity_to_steps(angular_velocity);
        let _ = self.puller.set_speed(steps_per_second);
    }
}

/// Implement machine connection helpers (without emit)
impl Winder2 {
    pub fn reverse_connect(&mut self) {
        let machine_identification_unique = self.machine_identification_unique.clone();
        if let Some(connected) = &self.connected_buffer {
            if let Some(buffer_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut buffer = buffer_arc.lock().await;
                    if buffer.connected_winder.is_none() {
                        buffer.set_connected_winder(machine_identification_unique);
                    }
                };
                smol::spawn(future).detach();
            }
        }
    }
}

// Enums
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Winder2Mode {
    Standby,
    Hold,
    Pull,
    Wind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpoolMode {
    Standby,
    Hold,
    Wind,
}

impl From<Winder2Mode> for SpoolMode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Self::Standby,
            Winder2Mode::Hold => Self::Hold,
            Winder2Mode::Pull => Self::Hold,
            Winder2Mode::Wind => Self::Wind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraverseMode {
    Standby,
    Hold,
    Traverse,
}

impl From<Winder2Mode> for TraverseMode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Self::Standby,
            Winder2Mode::Hold => Self::Hold,
            Winder2Mode::Pull => Self::Hold,
            Winder2Mode::Wind => Self::Traverse,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullerMode {
    Standby,
    Hold,
    Pull,
}

impl From<Winder2Mode> for PullerMode {
    fn from(mode: Winder2Mode) -> Self {
        match mode {
            Winder2Mode::Standby => Self::Standby,
            Winder2Mode::Hold => Self::Hold,
            Winder2Mode::Pull => Self::Pull,
            Winder2Mode::Wind => Self::Pull,
        }
    }
}

impl std::fmt::Display for Winder2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Winder2")
    }
}
