// Core functionality
pub mod act;
pub mod api;
pub mod emit;
pub mod new;

// Organized submodules
pub mod controllers;
pub mod features;

use units::Length;
use units::f64::ThermodynamicTemperature;
use units::thermodynamic_temperature::degree_celsius;

mod gluetex_imports {
    pub use super::api::GluetexNamespace;
    pub use super::api::SpoolAutomaticActionMode;
    pub use super::controllers::addon_motor_controller::AddonMotorController;
    pub use super::controllers::puller_speed_controller::PullerSpeedController;
    pub use super::controllers::slave_puller_speed_controller::SlavePullerSpeedController;
    pub use super::controllers::spool_speed_controller::SpoolSpeedController;
    pub use super::controllers::temperature_controller::TemperatureController;
    pub use super::controllers::tension_arm::TensionArm;
    pub use super::controllers::traverse_controller::TraverseController;
    pub use control_core::converters::angular_step_converter::AngularStepConverter;
    pub use ethercat_hal::io::{
        analog_input::AnalogInput, digital_input::DigitalInput, digital_output::DigitalOutput,
        stepper_velocity_el70x1::StepperVelocityEL70x1, temperature_input::TemperatureInput,
    };
    pub use smol::lock::RwLock;
    pub use std::{fmt::Debug, sync::Weak, time::Instant};

    pub use crate::buffer1::BufferV1;
    pub use units::ConstZero;
    pub use units::{
        Angle, angle::degree, length::meter, length::millimeter, velocity::meter_per_second,
    };
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

#[derive(Debug, Clone)]
pub struct TensionArmMonitorConfig {
    pub enabled: bool,
    pub min_angle: Angle,
    pub max_angle: Angle,
}

impl Default for TensionArmMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_angle: Angle::new::<degree>(10.0),
            max_angle: Angle::new::<degree>(170.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SleepTimerConfig {
    pub enabled: bool,
    pub timeout_seconds: u64,
}

impl Default for SleepTimerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            timeout_seconds: 900, // 15 minutes
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OrderInfo {
    pub order_number: String,
    pub serial_number: String,
    pub product_description: String,
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
    pub addon_motor_3_endstop: DigitalInput,
    pub addon_motor_3_analog_input: AnalogInput,
    pub addon_motor_4: StepperVelocityEL70x1,
    pub addon_motor_5: StepperVelocityEL70x1,

    // controllers
    pub traverse_controller: TraverseController,
    pub traverse_end_stop: DigitalInput,

    // addon motor controllers
    pub addon_motor_3_controller: AddonMotorController,
    pub addon_motor_4_controller: AddonMotorController,
    pub addon_motor_5_controller: AddonMotorController,
    /// Last time addon motor 3 was synced (for distance tracking)
    pub addon_motor_3_last_sync: Instant,

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
    last_state_emit: Instant,
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
    /// User preference for whether slave puller feature is enabled
    /// This is independent of mode-based enabling
    pub slave_puller_user_enabled: bool,

    // addon tension arm (independent tension arm on Role 9)
    pub addon_tension_arm: TensionArm,

    // optris temperature sensors (analog voltage inputs)
    pub optris_1: AnalogInput,
    pub optris_2: AnalogInput,

    // tension arm monitoring
    pub tension_arm_monitor_config: TensionArmMonitorConfig,
    pub tension_arm_monitor_triggered: bool,

    // sleep timer
    pub sleep_timer_config: SleepTimerConfig,
    pub last_activity_time: Instant,
    pub sleep_timer_triggered: bool,

    // order information
    pub order_info: OrderInfo,

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

        // Calculate distance moved since last sync
        let puller_speed = self.puller_speed_controller.last_speed;
        let dt = t.duration_since(self.addon_motor_3_last_sync).as_secs_f64();
        let distance_moved =
            Length::new::<meter>(puller_speed.get::<meter_per_second>() * dt).abs();

        // Check analog input for endstop (voltage < 1V means endstop is hit)
        // Endstop shows ~10V when not detected, ~0V when detected
        let endstop_hit = match self.addon_motor_3_analog_input.get_physical() {
            ethercat_hal::io::analog_input::physical::AnalogInputValue::Potential(voltage) => {
                use units::electric_potential::volt;
                voltage.get::<volt>() < 1.0
            }
            _ => false, // Current-based inputs are not expected here
        };

        self.addon_motor_3_controller.sync_motor_speed(
            &mut self.addon_motor_3,
            puller_angular_velocity,
            Some(endstop_hit),
            distance_moved,
        );

        self.addon_motor_3_last_sync = t;
    }

    /// Sync addon motor 4 speed based on puller angular velocity and ratio
    /// called by `act`
    pub fn sync_addon_motor_4_speed(&mut self, t: Instant) {
        let puller_angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
        self.addon_motor_4_controller.sync_motor_speed(
            &mut self.addon_motor_4,
            puller_angular_velocity,
            None,
            Length::ZERO,
        );
    }

    /// Sync addon motor 5 speed based on puller angular velocity and ratio
    /// called by `act`
    pub fn sync_addon_motor_5_speed(&mut self, t: Instant) {
        let puller_angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
        self.addon_motor_5_controller.sync_motor_speed(
            &mut self.addon_motor_5,
            puller_angular_velocity,
            None,
            Length::ZERO,
        );
    }

    /// Sync slave puller speed based on master puller speed and slave tension arm
    /// called by `act`
    pub fn sync_slave_puller_speed(&mut self, t: Instant) {
        // Get master puller speed as reference
        let master_speed = self.puller_speed_controller.get_target_speed();

        // Calculate slave speed based on slave tension arm
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

        // Only enable the slave puller if the user has enabled it in settings
        let should_enable_controller = self.slave_puller_user_enabled;

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
                    // Only enable speed controller if user wants it enabled
                    if should_enable_controller {
                        self.slave_puller_speed_controller.set_enabled(true);
                    }
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
                    // Only enable speed controller if user wants it enabled
                    if should_enable_controller {
                        self.slave_puller_speed_controller.set_enabled(true);
                    }
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

    /// Check all tension arm positions and trigger emergency stop if any are out of range
    pub fn check_tension_arm_monitor(&mut self) {
        // Only check if monitoring is enabled
        if !self.tension_arm_monitor_config.enabled {
            // Clear triggered flag if monitoring is disabled
            if self.tension_arm_monitor_triggered {
                self.tension_arm_monitor_triggered = false;
                self.emit_state();
            }
            return;
        }

        // Check all three tension arms
        let main_angle = self.tension_arm.get_angle();
        let slave_angle = self.addon_tension_arm.get_angle();
        let addon_angle = self.slave_tension_arm.get_angle();

        let min_angle = self.tension_arm_monitor_config.min_angle;
        let max_angle = self.tension_arm_monitor_config.max_angle;

        // Check if any tension arm is out of range
        let out_of_range = main_angle < min_angle
            || main_angle > max_angle
            || slave_angle < min_angle
            || slave_angle > max_angle
            || addon_angle < min_angle
            || addon_angle > max_angle;

        // If out of range, trigger emergency stop
        if out_of_range && !self.tension_arm_monitor_triggered {
            self.tension_arm_monitor_triggered = true;
            self.emergency_stop();
            self.emit_state();
            tracing::warn!(
                "Tension arm monitor triggered! Main: {:.1}°, Slave: {:.1}°, Addon: {:.1}° (limits: {:.1}°-{:.1}°)",
                main_angle.get::<degree>(),
                slave_angle.get::<degree>(),
                addon_angle.get::<degree>(),
                min_angle.get::<degree>(),
                max_angle.get::<degree>()
            );
        } else if !out_of_range && self.tension_arm_monitor_triggered {
            // Clear triggered flag if back in range
            self.tension_arm_monitor_triggered = false;
            self.emit_state();
            tracing::info!("Tension arm monitor cleared - all arms back in range");
        }
    }

    /// Emergency stop: stops all motors, heating, and sets machine to standby
    fn emergency_stop(&mut self) {
        // Stop all motors by setting mode to standby
        self.set_mode(&GluetexMode::Standby);

        // Disable heating
        self.heating_enabled = false;

        // Ensure all motors are disabled
        self.spool.set_enabled(false);
        self.puller.set_enabled(false);
        self.slave_puller.set_enabled(false);
        self.traverse.set_enabled(false);
        self.addon_motor_3.set_enabled(false);
        self.addon_motor_4.set_enabled(false);

        // Disable all controllers
        self.spool_speed_controller.set_enabled(false);
        self.puller_speed_controller.set_enabled(false);
        self.slave_puller_speed_controller.set_enabled(false);
    }

    /// Check if sleep timer has expired and trigger standby if needed
    pub fn check_sleep_timer(&mut self, now: Instant) {
        if !self.sleep_timer_config.enabled {
            self.sleep_timer_triggered = false;
            return;
        }

        let elapsed = now.duration_since(self.last_activity_time).as_secs();

        if elapsed >= self.sleep_timer_config.timeout_seconds && !self.sleep_timer_triggered {
            tracing::info!("Sleep timer expired - entering standby mode");
            self.sleep_timer_triggered = true;
            self.enter_sleep_mode();
        }
    }

    /// Enter sleep mode: similar to emergency stop but triggered by inactivity
    fn enter_sleep_mode(&mut self) {
        // Use emergency_stop to safely shut down everything
        self.emergency_stop();
        tracing::info!("Entered sleep mode due to inactivity");
    }

    /// Get remaining seconds on sleep timer
    pub fn get_sleep_timer_remaining_seconds(&self) -> u64 {
        if !self.sleep_timer_config.enabled {
            return 0;
        }

        // If timer has been triggered, keep it at 0
        if self.sleep_timer_triggered {
            return 0;
        }

        let elapsed = Instant::now()
            .duration_since(self.last_activity_time)
            .as_secs();

        if elapsed >= self.sleep_timer_config.timeout_seconds {
            0
        } else {
            self.sleep_timer_config.timeout_seconds - elapsed
        }
    }

    /// Reset the sleep timer (mark activity)
    pub fn reset_sleep_timer(&mut self) {
        self.last_activity_time = Instant::now();
        self.sleep_timer_triggered = false;
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
