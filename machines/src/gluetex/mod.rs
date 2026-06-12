// Core functionality
pub mod act;
pub mod api;
pub mod emit;
pub mod new;

#[cfg(feature = "gluetex-mock")]
pub mod mock;

// Organized submodules
pub mod controllers;
pub mod features;
pub mod monitoring;
pub mod safety;
pub mod state_from_domain;
pub mod sync;

use units::Length;
use units::f64::ThermodynamicTemperature;
use units::thermodynamic_temperature::degree_celsius;

mod gluetex_imports {
    pub use super::api::GluetexNamespace;
    pub use super::api::SpoolAutomaticActionMode;
    pub use super::controllers::heating::{HeatingBank, TemperatureController};
    pub use super::controllers::line::PullerSpeedController;
    pub use super::controllers::line::SlavePullerSpeedController;
    pub use super::controllers::line::ValveController;
    pub use super::controllers::steppers::{
        Stepper3Controller, Stepper4Controller, Stepper5Controller, Stepper5TensionController,
    };
    pub use super::controllers::tension::TensionArm;
    pub use super::controllers::winding::SpoolSpeedController;
    pub use super::controllers::winding::TraverseController;
    pub use control_core::converters::angular_step_converter::AngularStepConverter;
    pub use ethercat_hal::io::{
        analog_input::AnalogInput, digital_input::DigitalInput, digital_output::DigitalOutput,
        stepper_velocity_el70x1::StepperVelocityEL70x1, temperature_input::TemperatureInput,
    };
    pub use smol::lock::RwLock;
    pub use std::{fmt::Debug, sync::Weak, time::Instant};

    #[cfg(not(feature = "mock-machine"))]
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

// Re-export monitoring types for convenience
pub use monitoring::{
    SleepTimer, SleepTimerConfig, TensionArmMonitor, TensionArmMonitorConfig, VoltageMonitor,
    VoltageMonitorConfig,
};

#[derive(Debug, Clone, Default)]
pub struct OrderInfo {
    pub order_number: u32,
    pub serial_number: u32,
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

impl HeatingZone {
    pub const fn index(self) -> usize {
        match self {
            Self::Zone1 => 0,
            Self::Zone2 => 1,
            Self::Zone3 => 2,
            Self::Zone4 => 3,
            Self::Zone5 => 4,
            Self::Zone6 => 5,
        }
    }

    pub const ALL: [Self; 6] = [
        Self::Zone1,
        Self::Zone2,
        Self::Zone3,
        Self::Zone4,
        Self::Zone5,
        Self::Zone6,
    ];
}

pub use gluetex_imports::*;
use smol::channel::{Receiver, Sender};

use crate::{AsyncThreadMessage, Machine};
use crate::{
    MACHINE_GLUETEX_V1, MachineData, MachineMessage, VENDOR_QITECH,
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

    fn subscribed_to_machine(&mut self, uid: MachineIdentificationUnique) {
        self.connected_machine = Some(uid);
        self.emit_state();
    }

    fn unsubscribed_from_machine(&mut self, uid: MachineIdentificationUnique) {
        if self.connected_machine == Some(uid) {
            self.connected_machine = None;
            self.emit_state();
        }
    }

    fn receive_machines_data(&mut self, data: &MachineData) {
        _ = data;
    }
}

#[derive(Debug)]
pub struct Gluetex {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    main_sender: Option<Sender<AsyncThreadMessage>>,

    connected_machine: Option<MachineIdentificationUnique>,
    // drivers
    pub traverse: StepperVelocityEL70x1,
    pub puller: StepperVelocityEL70x1,
    pub spool: StepperVelocityEL70x1,
    pub winder_tension_arm: TensionArm,
    pub laser: DigitalOutput,
    pub status_out: DigitalOutput,
    pub extra_outputs: [DigitalOutput; 8],

    // addon motors
    pub stepper_3: StepperVelocityEL70x1,
    pub stepper_3_endstop: DigitalInput,
    pub stepper_3_analog_input: AnalogInput,
    pub stepper_4: StepperVelocityEL70x1,
    pub stepper_5: StepperVelocityEL70x1,

    // valve output and controller
    pub valve: DigitalOutput,
    pub valve_controller: ValveController,
    /// Last time valve was synced (for distance tracking)
    pub valve_last_sync: Instant,

    // controllers
    pub traverse_controller: TraverseController,
    pub traverse_end_stop: DigitalInput,
    /// Saved traverse position when going into hold mode
    /// This allows resuming exactly where it left off when returning to wind mode
    pub saved_traverse_position: Option<Length>,

    pub stepper_3_controller: Stepper3Controller,
    pub stepper_4_controller: Stepper4Controller,
    pub stepper_5_controller: Stepper5Controller,
    pub stepper_5_tension_controller: Stepper5TensionController,
    /// Last time addon motor 3 was synced (for distance tracking)
    pub stepper_3_last_sync: Instant,

    pub heaters: HeatingBank,

    // socketio
    namespace: GluetexNamespace,
    last_measurement_emit: Instant,
    last_state_emit: Instant,
    pub machine_identification_unique: MachineIdentificationUnique,

    // mode
    pub mode: GluetexMode,
    pub operation_mode: OperationMode,
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
    pub inlet_feeder_tension_arm: TensionArm,
    pub slave_puller_mode: PullerMode,
    /// User preference for whether slave puller feature is enabled
    /// This is independent of mode-based enabling
    pub slave_puller_user_enabled: bool,

    // TA tape feeder (independent tension arm on Role 9)
    pub tape_feeder_tension_arm: TensionArm,

    // optris temperature sensors (analog voltage inputs)
    pub optris_1: AnalogInput,
    pub optris_2: AnalogInput,
    // band monitoring (analog input converted to digital)
    pub bandueberwachung_input: AnalogInput,
    // unused analog inputs from EL7031 devices
    pub extra_analog_inputs: [AnalogInput; 3],

    // Monitoring systems
    pub winder_tension_arm_monitor: TensionArmMonitor,
    pub tape_feeder_tension_arm_monitor: TensionArmMonitor,
    pub inlet_feeder_tension_arm_monitor: TensionArmMonitor,
    pub optris_1_monitor: VoltageMonitor,
    pub optris_2_monitor: VoltageMonitor,
    pub bandueberwachung_triggered: bool,
    pub bandueberwachung_not_active_since: Option<std::time::Instant>,
    pub sleep_timer: SleepTimer,

    // Distance tracking for optris voltage delay
    /// Last recorded distance for optris 1 voltage delay calculation
    pub optris_1_last_distance_mm: f64,
    /// Last recorded distance for optris 2 voltage delay calculation
    pub optris_2_last_distance_mm: f64,

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

    /// Can wind capability check
    pub const fn can_wind(&self) -> bool {
        // Check if tension arm is zeroed and traverse is homed
        self.winder_tension_arm.zeroed
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
                    // Re-enable hardware in case shutdown_motors() disabled it (e.g. after safety stop)
                    self.spool.set_enabled(true);
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

    /// Record optris voltage readings with distance tracking for delayed readings
    /// Should be called once per control loop iteration
    pub fn record_optris_voltages(&mut self, _now: Instant) {
        // Read current voltages
        let optris_1_voltage = {
            use ethercat_hal::io::analog_input::physical::AnalogInputValue;
            use units::electric_potential::volt;
            match self.optris_1.get_physical() {
                AnalogInputValue::Potential(v) => v.get::<volt>(),
                _ => 0.0,
            }
        };

        let optris_2_voltage = {
            use ethercat_hal::io::analog_input::physical::AnalogInputValue;
            use units::electric_potential::volt;
            match self.optris_2.get_physical() {
                AnalogInputValue::Potential(v) => v.get::<volt>(),
                _ => 0.0,
            }
        };

        // Calculate distance moved since last recording for optris 1
        let current_distance_1 = self
            .spool_automatic_action
            .progress
            .get::<units::length::meter>()
            * 1000.0;
        let distance_moved_1 = (current_distance_1 - self.optris_1_last_distance_mm).max(0.0);
        self.optris_1_last_distance_mm = current_distance_1;
        self.optris_1_monitor
            .record_voltage(optris_1_voltage, distance_moved_1);

        // Calculate distance moved since last recording for optris 2
        let current_distance_2 = self
            .spool_automatic_action
            .progress
            .get::<units::length::meter>()
            * 1000.0;
        let distance_moved_2 = (current_distance_2 - self.optris_2_last_distance_mm).max(0.0);
        self.optris_2_last_distance_mm = current_distance_2;
        self.optris_2_monitor
            .record_voltage(optris_2_voltage, distance_moved_2);
    }

    /// Get remaining seconds on sleep timer
    pub fn get_sleep_timer_remaining_seconds(&self) -> u64 {
        self.sleep_timer.get_remaining_seconds(self.operation_mode)
    }

    /// Reset the sleep timer (mark activity)
    pub fn reset_sleep_timer(&mut self) {
        self.sleep_timer.reset();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GluetexMode {
    Standby,
    Hold,
    Pull,
    Wind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    /// Setup mode: safety monitoring is paused to allow setup movements
    Setup,
    /// Production mode: full operation with all safety monitoring active
    Production,
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
