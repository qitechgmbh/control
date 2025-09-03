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

#[cfg(feature = "mock-machine")]
pub mod mock;

#[cfg(feature = "mock-machine")]
mod winder2_imports {
    pub use super::api::SpoolAutomaticActionMode;
    pub use std::time::Instant;
    pub use uom::si::f64::Length;
}

#[cfg(not(feature = "mock-machine"))]
mod winder2_imports {
    pub use super::api::SpoolAutomaticActionMode;
    pub use super::api::Winder2Namespace;
    pub use super::puller_speed_controller::PullerSpeedController;
    pub use super::spool_speed_controller::SpoolSpeedController;
    pub use super::tension_arm::TensionArm;
    pub use super::traverse_controller::TraverseController;
    pub use control_core::{
        converters::angular_step_converter::AngularStepConverter,
        machines::{
            connection::{CrossConnectableMachine, MachineCrossConnection},
            identification::{MachineIdentification, MachineIdentificationUnique},
            manager::MachineManager,
        },
    };
    pub use control_core_derive::Machine;
    pub use ethercat_hal::io::{
        digital_input::DigitalInput, digital_output::DigitalOutput,
        stepper_velocity_el70x1::StepperVelocityEL70x1,
    };
    pub use smol::lock::RwLock;
    pub use std::{fmt::Debug, sync::Weak, time::Instant};
    pub use uom::si::f64::Length;

    pub use crate::machines::{MACHINE_WINDER_V1, VENDOR_QITECH, buffer1::BufferV1};
    pub use uom::ConstZero;
    pub use uom::si::{length::meter, length::millimeter, velocity::meter_per_second};
}

pub use winder2_imports::*;

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
            progress: Length::new::<uom::si::length::meter>(0.0),
            progress_last_check: Instant::now(),
            target_length: Length::new::<uom::si::length::meter>(0.0),
            mode: SpoolAutomaticActionMode::default(),
        }
    }
}

#[cfg(not(feature = "mock-machine"))]
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
    pub connected_buffer: MachineCrossConnection<Winder2, BufferV1>,
    pub connected_laser: Option<ConnectedMachine<Weak<Mutex<LaserMachine>>>>,

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

#[cfg(not(feature = "mock-machine"))]
impl CrossConnectableMachine<Winder2, BufferV1> for Winder2 {
    fn get_cross_connection(&mut self) -> &mut MachineCrossConnection<Winder2, BufferV1> {
        &mut self.connected_buffer
    }
}

#[cfg(not(feature = "mock-machine"))]
impl Winder2 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_WINDER_V1,
    };

    /// Validates that traverse limits maintain proper constraints:
    /// - Inner limit must be smaller than outer limit
    /// - At least 0.9mm difference between inner and outer limits
    fn validate_traverse_limits(inner: Length, outer: Length) -> bool {
        outer > inner + Length::new::<millimeter>(0.9)
    }

    pub fn traverse_set_limit_inner(&mut self, limit: f64) {
        let new_inner = Length::new::<millimeter>(limit);
        let current_outer = self.traverse_controller.get_limit_outer();

        // Validate the new inner limit against current outer limit
        if !Self::validate_traverse_limits(new_inner, current_outer) {
            // Don't update if validation fails - keep the current value
            return;
        }
        self.traverse_controller.set_limit_inner(new_inner);
        self.emit_state();
    }

    pub fn traverse_set_limit_outer(&mut self, limit: f64) {
        let new_outer = Length::new::<millimeter>(limit);
        let current_inner = self.traverse_controller.get_limit_inner();

        // Validate the new outer limit against current inner limit
        if !Self::validate_traverse_limits(current_inner, new_outer) {
            // Don't update if validation fails - keep the current value
            return;
        }

        self.traverse_controller.set_limit_outer(new_outer);
        self.emit_state();
    }

    pub fn traverse_set_step_size(&mut self, step_size: f64) {
        let step_size = Length::new::<millimeter>(step_size);
        self.traverse_controller.set_step_size(step_size);
        self.emit_state();
    }

    pub fn traverse_set_padding(&mut self, padding: f64) {
        let padding = Length::new::<millimeter>(padding);
        self.traverse_controller.set_padding(padding);
        self.emit_state();
    }

    pub fn traverse_goto_limit_inner(&mut self) {
        if self.can_go_in() {
            self.traverse_controller.goto_limit_inner();
        }
        self.emit_state();
    }

    pub fn traverse_goto_limit_outer(&mut self) {
        if self.can_go_out() {
            self.traverse_controller.goto_limit_outer();
        }
        self.emit_state();
    }

    pub fn traverse_goto_home(&mut self) {
        if self.can_go_home() {
            self.traverse_controller.goto_home();
        }
        self.emit_state();
    }

    pub fn emit_live_values(&mut self) {
        let angle_deg = self.tension_arm.get_angle().get::<degree>();

        // Wrap [270;<360] to [-90; 0]
        // This is done to reduce flicker in the graphs around the zero point
        let angle_deg = if angle_deg >= 270.0 {
            angle_deg - 360.0
        } else {
            angle_deg
        };

        // Calculate puller speed from current motor steps
        let steps_per_second = self.puller.get_speed();
        let angular_velocity = self
            .puller_speed_controller
            .converter
            .steps_to_angular_velocity(steps_per_second as f64);
        let puller_speed = self
            .puller_speed_controller
            .angular_velocity_to_speed(angular_velocity);

        // Calculate spool RPM from current motor steps
        let spool_rpm = self
            .spool_step_converter
            .steps_to_angular_velocity(self.spool.get_speed() as f64)
            .get::<revolution_per_minute>();

        let live_values = LiveValuesEvent {
            traverse_position: self
                .traverse_controller
                .get_current_position()
                .map(|x| x.get::<millimeter>()),
            puller_speed: puller_speed.get::<meter_per_minute>(),
            spool_rpm,
            tension_arm_angle: angle_deg,
            spool_progress: self.spool_automatic_action.progress.get::<meter>(),
        };

        let event = live_values.build();
        self.namespace.emit(Winder2Events::LiveValues(event));
    }

    pub fn build_state_event(&mut self) -> StateEvent {
        StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            traverse_state: TraverseState {
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
                can_go_in: self.can_go_in(),
                can_go_out: self.can_go_out(),
                can_go_home: self.can_go_home(),
            },
            puller_state: PullerState {
                regulation: self.puller_speed_controller.regulation_mode.clone(),
                target_speed: self
                    .puller_speed_controller
                    .target_speed
                    .get::<meter_per_minute>(),
                target_diameter: self
                    .puller_speed_controller
                    .target_diameter
                    .get::<millimeter>(),
                forward: self.puller_speed_controller.forward,
            },
            mode_state: ModeState {
                mode: self.mode.clone().into(),
                can_wind: self.can_wind(),
            },
            tension_arm_state: TensionArmState {
                zeroed: self.tension_arm.zeroed,
            },
            spool_speed_controller_state: SpoolSpeedControllerState {
                regulation_mode: self.spool_speed_controller.get_type().clone(),
                minmax_min_speed: self
                    .spool_speed_controller
                    .get_minmax_min_speed()
                    .get::<revolution_per_minute>(),
                minmax_max_speed: self
                    .spool_speed_controller
                    .get_minmax_max_speed()
                    .get::<revolution_per_minute>(),
                adaptive_tension_target: self.spool_speed_controller.get_adaptive_tension_target(),
                adaptive_radius_learning_rate: self
                    .spool_speed_controller
                    .get_adaptive_radius_learning_rate(),
                adaptive_max_speed_multiplier: self
                    .spool_speed_controller
                    .get_adaptive_max_speed_multiplier(),
                adaptive_acceleration_factor: self
                    .spool_speed_controller
                    .get_adaptive_acceleration_factor(),
                adaptive_deacceleration_urgency_multiplier: self
                    .spool_speed_controller
                    .get_adaptive_deacceleration_urgency_multiplier(),
            },
            spool_automatic_action_state: SpoolAutomaticActionState {
                spool_required_meters: self.spool_automatic_action.target_length.get::<meter>(),
                spool_automatic_action_mode: self.spool_automatic_action.mode.clone(),
            },
            connected_machine_state: ConnectedMachineState {
                machine_identification_unique: self.connected_buffer.as_ref().map(
                    |connected_machine| {
                        ConnectedMachineData::from(connected_machine).machine_identification_unique
                    },
                ),
                is_available: self
                    .connected_buffer
                    .as_ref()
                    .map(|connected_machine| {
                        ConnectedMachineData::from(connected_machine).is_available
                    })
                    .unwrap_or(false),
            },
            connected_laser_state: ConnectedMachineState {
                machine_identification_unique: self.connected_laser.as_ref().map(
                    |connected_machine| {
                        ConnectedMachineData::from(connected_machine)
                            .machine_identification_unique
                            .clone()
                    },
                ),
                is_available: self
                    .connected_laser
                    .as_ref()
                    .map(|connected_machine| {
                        ConnectedMachineData::from(connected_machine).is_available
                    })
                    .unwrap_or(false),
            },
            pid_settings: PidSettingsStates {
                speed: PidSettings {
                    ki: 0.1,
                    kp: 0.0,
                    kd: 0.2,
                    dead: 0.0,
                },
            },
        }
    }

    pub fn emit_state(&mut self) {
        let state_event = self.build_state_event();
        let event = state_event.build();
        self.namespace.emit(Winder2Events::State(event));
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
            && self.mode != Winder2Mode::Wind
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
            && self.mode != Winder2Mode::Wind
    }

    /// Can go home capability check
    pub fn can_go_home(&self) -> bool {
        // Check if not in standby, not traversing
        // Allow going home even when going in or out
        self.traverse_mode != TraverseMode::Standby
            && !self.traverse_controller.is_going_home()
            && !self.traverse_controller.is_traversing()
            && self.mode != Winder2Mode::Wind
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

    pub fn puller_set_regulation(&mut self, puller_regulation_mode: PullerRegulationMode) {
        self.puller_speed_controller
            .set_regulation_mode(puller_regulation_mode);
        self.emit_state();
    }

    /// Set target speed in m/min
    pub fn puller_set_target_speed(&mut self, target_speed: f64) {
        // Convert m/min to velocity
        let target_speed = Velocity::new::<meter_per_minute>(target_speed);
        self.puller_speed_controller.set_target_speed(target_speed);
        self.emit_state();
    }

    /// Set target diameter in mm
    pub fn puller_set_target_diameter(&mut self, target_diameter: f64) {
        // Convert m/min to velocity
        let target_diameter = Length::new::<millimeter>(target_diameter);
        self.puller_speed_controller
            .set_target_diameter(target_diameter);
        self.emit_state();
    }

    /// Set forward direction
    pub fn puller_set_forward(&mut self, forward: bool) {
        self.puller_speed_controller.set_forward(forward);
        self.emit_state();
    }

    // Spool Speed Controller API methods
    pub fn spool_set_regulation_mode(
        &mut self,
        regulation_mode: spool_speed_controller::SpoolSpeedControllerType,
    ) {
        self.spool_speed_controller.set_type(regulation_mode);
        self.emit_state();
    }

    /// Set minimum speed for minmax mode in RPM
    pub fn spool_set_minmax_min_speed(&mut self, min_speed_rpm: f64) {
        let min_speed = uom::si::f64::AngularVelocity::new::<revolution_per_minute>(min_speed_rpm);
        if let Err(e) = self.spool_speed_controller.set_minmax_min_speed(min_speed) {
            tracing::error!("Failed to set spool min speed: {:?}", e);
        }
        self.emit_state();
    }

    /// Set maximum speed for minmax mode in RPM
    pub fn spool_set_minmax_max_speed(&mut self, max_speed_rpm: f64) {
        let max_speed = uom::si::f64::AngularVelocity::new::<revolution_per_minute>(max_speed_rpm);
        if let Err(e) = self.spool_speed_controller.set_minmax_max_speed(max_speed) {
            tracing::error!("Failed to set spool max speed: {:?}", e);
        }
        self.emit_state();
    }

    /// Set tension target for adaptive mode (0.0-1.0)
    pub fn spool_set_adaptive_tension_target(&mut self, tension_target: f64) {
        self.spool_speed_controller
            .set_adaptive_tension_target(tension_target);
        self.emit_state();
    }

    /// Set radius learning rate for adaptive mode
    pub fn spool_set_adaptive_radius_learning_rate(&mut self, radius_learning_rate: f64) {
        self.spool_speed_controller
            .set_adaptive_radius_learning_rate(radius_learning_rate);
        self.emit_state();
    }

    /// Set max speed multiplier for adaptive mode
    pub fn spool_set_adaptive_max_speed_multiplier(&mut self, max_speed_multiplier: f64) {
        self.spool_speed_controller
            .set_adaptive_max_speed_multiplier(max_speed_multiplier);
        self.emit_state();
    }

    /// Set acceleration factor for adaptive mode
    pub fn spool_set_adaptive_acceleration_factor(&mut self, acceleration_factor: f64) {
        self.spool_speed_controller
            .set_adaptive_acceleration_factor(acceleration_factor);
        self.emit_state();
    }

    /// Set deacceleration urgency multiplier for adaptive mode
    pub fn spool_set_adaptive_deacceleration_urgency_multiplier(
        &mut self,
        deacceleration_urgency_multiplier: f64,
    ) {
        self.spool_speed_controller
            .set_adaptive_deacceleration_urgency_multiplier(deacceleration_urgency_multiplier);
        self.emit_state();
    }
}

/// implement buffer connection
impl Winder2 {
    /// set connected buffer
    pub fn set_connected_buffer(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            BufferV1::MACHINE_IDENTIFICATION
        ) {
            return;
        }
        let machine_manager_arc = match self.machine_manager.upgrade() {
            Some(machine_manager_arc) => machine_manager_arc,
            None => return,
        };
        let machine_manager_guard = block_on(machine_manager_arc.read());
        let buffer_weak = machine_manager_guard.get_machine_weak(&machine_identification_unique);
        let buffer_weak = match buffer_weak {
            Some(buffer_weak) => buffer_weak,
            None => return,
        };
        let buffer_strong = match buffer_weak.upgrade() {
            Some(buffer_strong) => buffer_strong,
            None => return,
        };

        let buffer: Arc<Mutex<BufferV1>> = block_on(downcast_machine::<BufferV1>(buffer_strong))
            .expect("failed downcasting machine");

        let machine = Arc::downgrade(&buffer);

        self.connected_buffer = Some(ConnectedMachine {
            machine_identification_unique,
            machine,
        });
        self.emit_state();

        self.reverse_connect();
    }

    /// disconnect buffer
    pub fn disconnect_buffer(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            BufferV1::MACHINE_IDENTIFICATION
        ) {
            return;
        }
        if let Some(connected) = &self.connected_buffer {
            if let Some(buffer_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut buffer = buffer_arc.lock().await;
                    if buffer.connected_winder.is_some() {
                        buffer.connected_winder = None;
                        buffer.emit_state();
                    }
                };
                smol::spawn(future).detach();
            }
        }
        self.connected_buffer = None;
        self.emit_state();
    }

    /// initiate connection from buffer to winder
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

/// implement laser connection
impl Winder2 {
    /// set connected laser
    pub fn set_connected_laser(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            LaserMachine::MACHINE_IDENTIFICATION
        ) {
            tracing::trace!("Setting Connected Laser | did not match ID");
            return;
        }
        let machine_manager_arc = match self.machine_manager.upgrade() {
            Some(machine_manager_arc) => machine_manager_arc,
            None => {
                tracing::trace!("Setting Connected Laser | Failed to upgrade machine manager");
                return;
            }
        };
        let machine_manager_guard = block_on(machine_manager_arc.read());
        let laser_weak = machine_manager_guard.get_serial_weak(&machine_identification_unique);
        let laser_weak = match laser_weak {
            Some(laser_weak) => laser_weak,
            None => {
                tracing::trace!("Setting Connected Laser | Failed to get serial weak");
                return;
            }
        };
        let laser_strong = match laser_weak.upgrade() {
            Some(laser_strong) => laser_strong,
            None => {
                tracing::trace!("Setting Connected Laser | Failed to upgrade to Strong");
                return;
            }
        };

        let laser: Arc<Mutex<LaserMachine>> =
            block_on(downcast_machine::<LaserMachine>(laser_strong))
                .expect("failed downcasting machine");

        let machine = Arc::downgrade(&laser);

        self.connected_laser = Some(ConnectedMachine {
            machine_identification_unique,
            machine: machine.clone(),
        });

        self.emit_state();

        self.reverse_connect_laser();
    }

    /// disconnect laser
    pub fn disconnect_laser(&mut self, machine_identification_unique: MachineIdentificationUnique) {
        if !matches!(
            machine_identification_unique.machine_identification,
            LaserMachine::MACHINE_IDENTIFICATION
        ) {
            return;
        }
        if let Some(connected) = &self.connected_laser {
            if let Some(laser_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut laser = laser_arc.lock().await;
                    if laser.connected_winder.is_some() {
                        laser.connected_winder = None;
                        laser.emit_state();
                    }
                };
                smol::spawn(future).detach();
            }
        }
        self.connected_laser = None;
        self.emit_state();
    }

    /// initiate connection from laser to winder
    pub fn reverse_connect_laser(&mut self) {
        let machine_identification_unique = self.machine_identification_unique.clone();
        if let Some(connected) = &self.connected_laser {
            if let Some(laser_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut laser = laser_arc.lock().await;
                    if laser.connected_winder.is_none() {
                        laser.set_connected_winder(machine_identification_unique);
                    }
                };
                smol::spawn(future).detach();
            }
        }
    }
}

impl Winder2 {
    pub fn configure_speed_pid(&mut self, settings: PidSettings) {
        // Implement pid to controll speed of winder
        self.puller_speed_controller
            .pid
            .configure(settings.ki, settings.kp, settings.kd);
        self.emit_state();
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[cfg(not(feature = "mock-machine"))]
impl std::fmt::Display for Winder2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Winder2")
    }
}

#[cfg(not(feature = "mock-machine"))]
#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::{f64::Length, length::millimeter};

    #[test]
    fn test_validate_traverse_limits() {
        // Test case 1: Valid limits with exactly 1.0mm difference (should pass)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(16.0);
        assert!(Winder2::validate_traverse_limits(inner, outer));

        // Test case 2: Invalid limits with exactly 0.9mm difference (should fail)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.9);
        assert!(!Winder2::validate_traverse_limits(inner, outer));

        // Test case 3: Invalid limits with less than 0.9mm difference (should fail)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.5);
        assert!(!Winder2::validate_traverse_limits(inner, outer));

        // Test case 4: Invalid limits where inner equals outer (should fail)
        let inner = Length::new::<millimeter>(20.0);
        let outer = Length::new::<millimeter>(20.0);
        assert!(!Winder2::validate_traverse_limits(inner, outer));

        // Test case 5: Invalid limits where inner is greater than outer (should fail)
        let inner = Length::new::<millimeter>(25.0);
        let outer = Length::new::<millimeter>(20.0);
        assert!(!Winder2::validate_traverse_limits(inner, outer));

        // Test case 6: Valid limits with large difference (should pass)
        let inner = Length::new::<millimeter>(10.0);
        let outer = Length::new::<millimeter>(80.0);
        assert!(Winder2::validate_traverse_limits(inner, outer));

        // Test case 7: Edge case - exactly 0.91mm difference (should pass)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.91);
        assert!(Winder2::validate_traverse_limits(inner, outer));

        // Test case 8: Edge case - exactly 0.89mm difference (should fail)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.89);
        assert!(!Winder2::validate_traverse_limits(inner, outer));
    }
}
