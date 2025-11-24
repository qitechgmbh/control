pub mod act;
pub mod adaptive_spool_speed_controller;
pub mod addon_motor_controller;
pub mod api;
pub mod clamp_revolution;
pub mod emit;
pub mod filament_tension;
pub mod minmax_spool_speed_controller;
pub mod new;
pub mod puller_speed_controller;
pub mod slave_puller_speed_controller;
pub mod spool_speed_controller;
pub mod temperature_controller;
pub mod tension_arm;
pub mod traverse_controller;

use units::Length;
use units::f64::ThermodynamicTemperature;
use units::thermodynamic_temperature::degree_celsius;

mod gluetex_imports {
    pub use super::addon_motor_controller::AddonMotorController;
    pub use super::api::GluetexNamespace;
    pub use super::api::SpoolAutomaticActionMode;
    pub use super::puller_speed_controller::PullerSpeedController;
    pub use super::slave_puller_speed_controller::SlavePullerSpeedController;
    pub use super::spool_speed_controller::SpoolSpeedController;
    pub use super::temperature_controller::TemperatureController;
    pub use super::tension_arm::TensionArm;
    pub use super::traverse_controller::TraverseController;
    pub use control_core::converters::angular_step_converter::AngularStepConverter;
    pub use ethercat_hal::io::{
        digital_input::DigitalInput, digital_output::DigitalOutput,
        stepper_velocity_el70x1::StepperVelocityEL70x1, temperature_input::TemperatureInput,
    };
    pub use smol::lock::RwLock;
    pub use std::{fmt::Debug, sync::Weak, time::Instant};

    pub use crate::buffer1::BufferV1;
    pub use units::ConstZero;
    pub use units::{length::meter, length::millimeter, velocity::meter_per_second};
}

#[derive(Debug, Clone)]
pub struct Heating {
    pub temperature: ThermodynamicTemperature,
    pub heating: bool,
    pub target_temperature: ThermodynamicTemperature,
    pub wiring_error: bool,
}

impl Default for Heating {
    fn default() -> Self {
        Self {
            temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
            heating: false,
            target_temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
            wiring_error: false,
        }
    }
}

pub enum HeatingZone {
    Zone1,
    Zone2,
    Zone3,
    Zone4,
    Zone5,
    Zone6,
}

pub use gluetex_imports::*;
use smol::channel::{Receiver, Sender};

use crate::{AsyncThreadMessage, Machine};
use crate::{
    MACHINE_GLUETEX_V1, MachineConnection, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

#[derive(Debug)]
pub struct SpoolAutomaticAction {
    pub progress: Length,
    progress_last_check: Instant,
    pub target_length: Length,
    pub mode: SpoolAutomaticActionMode,
}

impl Default for SpoolAutomaticAction {
    fn default() -> Self {
        SpoolAutomaticAction {
            progress: Length::new::<meter>(0.0),
            progress_last_check: Instant::now(),
            target_length: Length::new::<meter>(0.0),
            mode: SpoolAutomaticActionMode::default(),
        }
    }
}

impl Machine for Gluetex {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

#[derive(Debug)]
pub struct Gluetex {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    main_sender: Option<Sender<AsyncThreadMessage>>,

    connected_machines: Vec<MachineConnection>,
    max_connected_machines: usize,
    // drivers
    pub traverse: StepperVelocityEL70x1,
    pub puller: StepperVelocityEL70x1,
    pub spool: StepperVelocityEL70x1,
    pub tension_arm: TensionArm,
    pub laser: DigitalOutput,

    // addon motors
    pub addon_motor_3: StepperVelocityEL70x1,
    pub addon_motor_4: StepperVelocityEL70x1,

    // controllers
    pub traverse_controller: TraverseController,
    pub traverse_end_stop: DigitalInput,

    // addon motor controllers
    pub addon_motor_3_controller: AddonMotorController,
    pub addon_motor_4_controller: AddonMotorController,

    // temperature controllers (PID-controlled heaters with temperature sensors)
    pub temperature_controller_1: TemperatureController,
    pub temperature_controller_2: TemperatureController,
    pub temperature_controller_3: TemperatureController,
    pub temperature_controller_4: TemperatureController,
    pub temperature_controller_5: TemperatureController,
    pub temperature_controller_6: TemperatureController,
    pub heating_enabled: bool,

    // socketio
    namespace: GluetexNamespace,
    last_measurement_emit: Instant,
    pub machine_identification_unique: MachineIdentificationUnique,

    // mode
    pub mode: GluetexMode,
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

    // slave puller (secondary puller with tension control)
    pub slave_puller: StepperVelocityEL70x1,
    pub slave_puller_speed_controller: SlavePullerSpeedController,
    pub slave_tension_arm: TensionArm,
    pub slave_puller_mode: PullerMode,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
}

impl Gluetex {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_GLUETEX_V1,
    };

    /// Validates that traverse limits maintain proper constraints:
    /// - Inner limit must be smaller than outer limit
    /// - At least 0.9mm difference between inner and outer limits
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

    /// Can wind capability check
    pub const fn can_wind(&self) -> bool {
        // Check if tension arm is zeroed and traverse is homed
        self.tension_arm.zeroed
            && self.traverse_controller.is_homed()
            && !self.traverse_controller.is_going_home()
    }

    /// Can go to inner limit capability check
    pub fn can_go_in(&self) -> bool {
        // Check if traverse is homed, not in standby, not traversing
        // Allow changing direction (even when going out)
        // Disallow when homing is in progress
        self.traverse_controller.is_homed()
            && self.traverse_mode != TraverseMode::Standby
            && !self.traverse_controller.is_going_in()
            && !self.traverse_controller.is_going_home()
            && !self.traverse_controller.is_traversing()
            && self.mode != GluetexMode::Wind
    }

    /// Can go to outer limit capability check
    pub fn can_go_out(&self) -> bool {
        // Check if traverse is homed, not in standby, not traversing
        // Allow changing direction (even when going in)
        // Disallow when homing is in progress
        self.traverse_controller.is_homed()
            && self.traverse_mode != TraverseMode::Standby
            && !self.traverse_controller.is_going_out()
            && !self.traverse_controller.is_going_home()
            && !self.traverse_controller.is_traversing()
            && self.mode != GluetexMode::Wind
    }

    /// Can go home capability check
    pub fn can_go_home(&self) -> bool {
        // Check if not in standby, not traversing
        // Allow going home even when going in or out
        self.traverse_mode != TraverseMode::Standby
            && !self.traverse_controller.is_going_home()
            && !self.traverse_controller.is_traversing()
            && self.mode != GluetexMode::Wind
    }

    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_spool_mode(&mut self, mode: &GluetexMode) {
        // Convert to `GluetexMode` to `SpoolMode`
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
                    self.spool.set_enabled(true);
                    // self.spool_speed_controller.reset();
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
                    // self.spool_speed_controller.reset();
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

    /// Apply the mode changes to the puller
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::puller_mode`]
    fn set_puller_mode(&mut self, mode: &GluetexMode) {
        // Convert to `GluetexMode` to `PullerMode`
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

    pub const fn stop_or_pull_spool_reset(&mut self, now: Instant) {
        self.spool_automatic_action.progress = Length::ZERO;
        self.spool_automatic_action.progress_last_check = now;
    }

    pub fn calculate_spool_auto_progress_(&mut self, now: Instant) {
        // Calculate time elapsed since last progress check (in minutes)

        let dt = now
            .duration_since(self.spool_automatic_action.progress_last_check)
            .as_secs_f64();

        // Calculate distance pulled during this time interval
        let meters_pulled_this_interval = Length::new::<meter>(
            self.puller_speed_controller
                .last_speed
                .get::<meter_per_second>()
                * dt,
        );

        // Update total meters pulled
        self.spool_automatic_action.progress += meters_pulled_this_interval.abs();
        self.spool_automatic_action.progress_last_check = now;
    }

    /// Implement Puller
    /// called by `act`
    pub fn sync_puller_speed(&mut self, t: Instant) {
        let angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
        let steps_per_second = self
            .puller_speed_controller
            .converter
            .angular_velocity_to_steps(angular_velocity);
        let _ = self.puller.set_speed(steps_per_second);
    }

    /// Sync addon motor 3 speed based on puller angular velocity and ratio
    /// called by `act`
    pub fn sync_addon_motor_3_speed(&mut self, t: Instant) {
        let puller_angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
        self.addon_motor_3_controller
            .sync_motor_speed(&mut self.addon_motor_3, puller_angular_velocity);
    }

    /// Sync addon motor 4 speed based on puller angular velocity and ratio
    /// called by `act`
    pub fn sync_addon_motor_4_speed(&mut self, t: Instant) {
        let puller_angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
        self.addon_motor_4_controller
            .sync_motor_speed(&mut self.addon_motor_4, puller_angular_velocity);
    }

    /// Sync slave puller speed based on master puller speed and slave tension arm
    /// called by `act`
    pub fn sync_slave_puller_speed(&mut self, t: Instant) {
        // Get master puller speed as reference
        let master_speed = self.puller_speed_controller.get_target_speed();

        // Calculate slave speed based on tension arm
        let slave_velocity = self.slave_puller_speed_controller.update_speed(
            t,
            master_speed,
            &self.slave_tension_arm,
        );

        // Apply direction
        let directed_velocity = if self.slave_puller_speed_controller.get_forward() {
            slave_velocity
        } else {
            -slave_velocity
        };

        // Convert to angular velocity then to steps
        let angular_velocity = self
            .slave_puller_speed_controller
            .velocity_to_angular_velocity(directed_velocity);

        let steps_per_second = self
            .slave_puller_speed_controller
            .converter
            .angular_velocity_to_steps(angular_velocity);

        let _ = self.slave_puller.set_speed(steps_per_second);
    }

    /// Apply the mode changes to the slave puller
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::slave_puller_mode`]
    fn set_slave_puller_mode(&mut self, mode: &GluetexMode) {
        // Convert to `GluetexMode` to `PullerMode`
        let mode: PullerMode = mode.clone().into();

        // Transition matrix
        match self.slave_puller_mode {
            PullerMode::Standby => match mode {
                PullerMode::Standby => {}
                PullerMode::Hold => {
                    // From [`PullerMode::Standby`] to [`PullerMode::Hold`]
                    self.slave_puller.set_enabled(true);
                }
                PullerMode::Pull => {
                    // From [`PullerMode::Standby`] to [`PullerMode::Pull`]
                    self.slave_puller.set_enabled(true);
                    self.slave_puller_speed_controller.set_enabled(true);
                }
            },
            PullerMode::Hold => match mode {
                PullerMode::Standby => {
                    // From [`PullerMode::Hold`] to [`PullerMode::Standby`]
                    self.slave_puller.set_enabled(false);
                }
                PullerMode::Hold => {}
                PullerMode::Pull => {
                    // From [`PullerMode::Hold`] to [`PullerMode::Pull`]
                    self.slave_puller_speed_controller.set_enabled(true);
                }
            },
            PullerMode::Pull => match mode {
                PullerMode::Standby => {
                    // From [`PullerMode::Pull`] to [`PullerMode::Standby`]
                    self.slave_puller.set_enabled(false);
                    self.slave_puller_speed_controller.set_enabled(false);
                }
                PullerMode::Hold => {
                    // From [`PullerMode::Pull`] to [`PullerMode::Hold`]
                    self.slave_puller_speed_controller.set_enabled(false);
                }
                PullerMode::Pull => {}
            },
        }

        // Update the internal state
        self.slave_puller_mode = mode;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GluetexMode {
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

impl From<GluetexMode> for SpoolMode {
    fn from(mode: GluetexMode) -> Self {
        match mode {
            GluetexMode::Standby => Self::Standby,
            GluetexMode::Hold => Self::Hold,
            GluetexMode::Pull => Self::Hold,
            GluetexMode::Wind => Self::Wind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraverseMode {
    Standby,
    Hold,
    Traverse,
}

impl From<GluetexMode> for TraverseMode {
    fn from(mode: GluetexMode) -> Self {
        match mode {
            GluetexMode::Standby => Self::Standby,
            GluetexMode::Hold => Self::Hold,
            GluetexMode::Pull => Self::Hold,
            GluetexMode::Wind => Self::Traverse,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullerMode {
    Standby,
    Hold,
    Pull,
}

impl From<GluetexMode> for PullerMode {
    fn from(mode: GluetexMode) -> Self {
        match mode {
            GluetexMode::Standby => Self::Standby,
            GluetexMode::Hold => Self::Hold,
            GluetexMode::Pull => Self::Pull,
            GluetexMode::Wind => Self::Pull,
        }
    }
}

impl std::fmt::Display for Gluetex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gluetex")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_traverse_limits() {
        // Test case 1: Valid limits with exactly 1.0mm difference (should pass)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(16.0);
        assert!(Gluetex::validate_traverse_limits(inner, outer));

        // Test case 2: Invalid limits with exactly 0.9mm difference (should fail)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.9);
        assert!(!Gluetex::validate_traverse_limits(inner, outer));

        // Test case 3: Invalid limits with less than 0.9mm difference (should fail)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.5);
        assert!(!Gluetex::validate_traverse_limits(inner, outer));

        // Test case 4: Invalid limits where inner equals outer (should fail)
        let inner = Length::new::<millimeter>(20.0);
        let outer = Length::new::<millimeter>(20.0);
        assert!(!Gluetex::validate_traverse_limits(inner, outer));

        // Test case 5: Invalid limits where inner is greater than outer (should fail)
        let inner = Length::new::<millimeter>(25.0);
        let outer = Length::new::<millimeter>(20.0);
        assert!(!Gluetex::validate_traverse_limits(inner, outer));

        // Test case 6: Valid limits with large difference (should pass)
        let inner = Length::new::<millimeter>(10.0);
        let outer = Length::new::<millimeter>(80.0);
        assert!(Gluetex::validate_traverse_limits(inner, outer));

        // Test case 7: Edge case - exactly 0.91mm difference (should pass)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.91);
        assert!(Gluetex::validate_traverse_limits(inner, outer));

        // Test case 8: Edge case - exactly 0.89mm difference (should fail)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.89);
        assert!(!Gluetex::validate_traverse_limits(inner, outer));
    }
}
