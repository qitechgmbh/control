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
    pub use units::f64::{AngularVelocity, Length, Velocity};
    pub use units::length::meter;
}

#[cfg(not(feature = "mock-machine"))]
mod winder2_imports {
    pub use super::api::SpoolAutomaticActionMode;
    pub use super::api::Winder2Namespace;
    pub use super::puller_speed_controller::PullerSpeedController;
    pub use super::spool_speed_controller::SpoolSpeedController;
    pub use super::tension_arm::TensionArm;
    pub use super::traverse_controller::TraverseController;
    pub use control_core::converters::angular_step_converter::AngularStepConverter;
    pub use ethercat_hal::devices::wago_modules::wago_750_467::{Wago750_467, Wago750_467Port};
    pub use ethercat_hal::io::analog_input::physical::AnalogInputValue;
    pub use ethercat_hal::io::{
        analog_input::AnalogInput, digital_output::DigitalOutput,
        stepper_velocity_wago_750_671::StepperVelocityWago750671,
        stepper_velocity_wago_750_671_traverse::StepperVelocityWago750671Traverse,
        stepper_velocity_wago_750_672::StepperVelocityWago750672,
    };
    pub use smol::channel::{Receiver, Sender};
    pub use smol::lock::RwLock;
    pub use std::{
        fmt::Debug,
        sync::Weak,
        time::{Duration, Instant},
    };

    pub use crate::buffer1::BufferV1;
    pub use crate::{AsyncThreadMessage, Machine};
    pub use units::ConstZero;
    pub use units::electric_potential::volt;
    pub use units::f64::Length;
    pub use units::{length::meter, length::millimeter, velocity::meter_per_second};
}

pub use winder2_imports::*;

#[cfg(not(feature = "mock-machine"))]
use crate::{
    MACHINE_WAGO_WINDER_V1, MachineData, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
#[cfg(not(feature = "mock-machine"))]
use units::angle::degree;
#[cfg(not(feature = "mock-machine"))]
use units::f64::{AngularVelocity, Velocity};

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

#[cfg(not(feature = "mock-machine"))]
impl Machine for WagoWinder {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }

    fn receive_machines_data(&mut self, data: &MachineData) {
        use MachineData::*;

        debug_assert!(self.puller_reference_machine.is_some());

        match data {
            Laser(state, live_values) => {
                let current = live_values.diameter;
                let target = state.laser_state.target_diameter;
                let lower = state.laser_state.lower_tolerance;
                let upper = state.laser_state.higher_tolerance;
                let last_speed = self.puller_speed_controller.last_speed;

                self.puller_speed_controller
                    .adaptive
                    .update_with_measurement(
                        current,
                        target,
                        lower,
                        upper,
                        last_speed,
                        Instant::now(),
                    );
            }
            None => tracing::error!("Received MachineData::None"),
        };
    }

    fn subscribed_to_machine(&mut self, uid: MachineIdentificationUnique) {
        self.puller_reference_machine = Some(uid);
        self.emit_state();
    }

    fn unsubscribed_from_machine(&mut self, uid: MachineIdentificationUnique) {
        if let Some(current_uid) = self.puller_reference_machine {
            if current_uid == uid {
                self.puller_reference_machine = None;
            }
        }

        self.emit_state();
    }
}

#[cfg(not(feature = "mock-machine"))]
#[derive(Debug)]
pub struct WagoWinder {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    main_sender: Option<Sender<AsyncThreadMessage>>,

    // drivers
    pub traverse: StepperVelocityWago750671Traverse,
    pub puller: StepperVelocityWago750672,
    pub spool: StepperVelocityWago750672,
    pub tension_arm: TensionArm,
    pub tension_arm_raw: AnalogInput,
    pub laser: DigitalOutput,

    // controllers
    pub traverse_controller: TraverseController,

    // socketio
    namespace: Winder2Namespace,
    last_measurement_emit: Instant,
    last_debug_signature: Option<String>,
    last_axis_status_signature: Option<String>,
    last_traverse_debug_raw_position: Option<i128>,
    last_traverse_debug_raw_delta: i128,
    last_control_loop_debug_emit: Instant,
    pub machine_identification_unique: MachineIdentificationUnique,

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
    pub spool_tension_blocked: bool,

    // control circuit puller
    pub puller_speed_controller: PullerSpeedController,

    // reference machine for adapative mode
    puller_reference_machine: Option<MachineIdentificationUnique>,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
}

#[cfg(not(feature = "mock-machine"))]
impl WagoWinder {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_WAGO_WINDER_V1,
    };

    fn actual_spool_angular_velocity(&self) -> AngularVelocity {
        self.spool_step_converter
            .steps_to_angular_velocity(self.spool.get_actual_speed_steps_per_second())
    }

    fn actual_puller_linear_speed(&self) -> Velocity {
        let angular_velocity = self
            .puller_speed_controller
            .converter
            .steps_to_angular_velocity(self.puller.get_actual_speed_steps_per_second());
        let motor_speed = self
            .puller_speed_controller
            .angular_velocity_to_speed(angular_velocity);
        motor_speed / self.puller_speed_controller.get_gear_ratio().multiplier()
    }

    fn tension_arm_in_spool_window(&self) -> bool {
        let angle_deg = self.tension_arm.get_angle().get::<degree>();
        (20.0..=90.0).contains(&angle_deg)
    }

    fn tension_arm_in_spool_restart_window(&self) -> bool {
        let angle_deg = self.tension_arm.get_angle().get::<degree>();
        (25.0..=85.0).contains(&angle_deg)
    }

    /// Validates that traverse limits maintain proper constraints:
    /// - Inner limit must be smaller than outer limit
    /// - At least 0.9mm difference between inner and outer limits
    fn validate_traverse_limits(inner: Length, outer: Length) -> bool {
        outer > inner + Length::new::<millimeter>(0.9)
    }

    pub fn sync_traverse_speed(&mut self) {
        let spool_speed = self.actual_spool_angular_velocity();
        let traverse_end_stop = self.traverse.get_s3_bit0();
        self.traverse_controller
            .update_speed(&mut self.traverse, traverse_end_stop, spool_speed)
    }

    pub fn current_axis_status_signature(&self) -> String {
        format!(
            "traverse_di1={}|traverse_ack={}|traverse_raw_pos={}|traverse_s1=0x{:02X}|traverse_s2=0x{:02X}|traverse_s3=0x{:02X}|traverse_c1=0x{:02X}|traverse_c2=0x{:02X}|traverse_c3=0x{:02X}|spool_di1={}|spool_ack={}|spool_raw_pos={}|spool_s1=0x{:02X}|spool_s2=0x{:02X}|spool_s3=0x{:02X}|spool_c1=0x{:02X}|spool_c2=0x{:02X}|spool_c3=0x{:02X}",
            self.traverse.get_s3_bit0(),
            self.traverse.get_s1_bit3_speed_mode_ack(),
            self.traverse.get_raw_position(),
            self.traverse.get_status_byte1(),
            self.traverse.get_status_byte2(),
            self.traverse.get_status_byte3(),
            self.traverse.get_control_byte1(),
            self.traverse.get_control_byte2(),
            self.traverse.get_control_byte3(),
            self.spool.get_s3_bit0(),
            self.spool.get_s1_bit3_speed_mode_ack(),
            self.spool.get_raw_position(),
            self.spool.get_status_byte1(),
            self.spool.get_status_byte2(),
            self.spool.get_status_byte3(),
            self.spool.get_control_byte1(),
            self.spool.get_control_byte2(),
            self.spool.get_control_byte3(),
        )
    }

    fn build_debug_key(&self) -> String {
        format!(
            "mode={:?}|spool_mode={:?}|traverse_mode={:?}|puller_mode={:?}|can_wind={}|can_go_in={}|can_go_out={}|can_go_home={}|traverse_state={}|traverse_cmd_sign={:?}|tension_zeroed={}|laser={}|spool_c1=0x{:02X}|spool_c2=0x{:02X}|spool_c3=0x{:02X}|spool_s1=0x{:02X}|spool_s2=0x{:02X}|spool_s3=0x{:02X}|spool_ack={}|spool_di1={}|traverse_c1=0x{:02X}|traverse_c2=0x{:02X}|traverse_c3=0x{:02X}|traverse_s1=0x{:02X}|traverse_s2=0x{:02X}|traverse_s3=0x{:02X}|traverse_ack={}|traverse_ref_ack={}|traverse_ref_ok={}|traverse_busy={}|traverse_di1={}|traverse_di2={}|puller_c1=0x{:02X}|puller_c2=0x{:02X}|puller_c3=0x{:02X}|puller_s1=0x{:02X}|puller_s2=0x{:02X}|puller_s3=0x{:02X}|puller_ack={}|puller_di1={}",
            self.mode,
            self.spool_mode,
            self.traverse_mode,
            self.puller_mode,
            self.can_wind(),
            self.can_go_in(),
            self.can_go_out(),
            self.can_go_home(),
            self.traverse_controller.debug_state(),
            self.traverse_controller.debug_homing_command_sign(),
            self.tension_arm.zeroed,
            self.laser.get(),
            self.spool.get_control_byte1(),
            self.spool.get_control_byte2(),
            self.spool.get_control_byte3(),
            self.spool.get_status_byte1(),
            self.spool.get_status_byte2(),
            self.spool.get_status_byte3(),
            self.spool.get_s1_bit3_speed_mode_ack(),
            self.spool.get_s3_bit0(),
            self.traverse.get_control_byte1(),
            self.traverse.get_control_byte2(),
            self.traverse.get_control_byte3(),
            self.traverse.get_status_byte1(),
            self.traverse.get_status_byte2(),
            self.traverse.get_status_byte3(),
            self.traverse.get_s1_bit3_speed_mode_ack(),
            self.traverse.get_s1_bit5_reference_mode_ack(),
            self.traverse.get_s2_reference_ok(),
            self.traverse.get_s2_busy(),
            self.traverse.get_s3_bit0(),
            self.traverse.get_s3_bit1(),
            self.puller.get_control_byte1(),
            self.puller.get_control_byte2(),
            self.puller.get_control_byte3(),
            self.puller.get_status_byte1(),
            self.puller.get_status_byte2(),
            self.puller.get_status_byte3(),
            self.puller.get_s1_bit3_speed_mode_ack(),
            self.puller.get_s3_bit0(),
        )
    }

    fn build_debug_snapshot(&self) -> String {
        let tension_voltage = match self.tension_arm.analog_input.get_physical() {
            AnalogInputValue::Potential(v) => v.get::<volt>(),
            _ => 0.0,
        };

        format!(
            "WagoWinder snapshot | mode={:?} spool_mode={:?} traverse_mode={:?} puller_mode={:?} can_wind={} can_go_in={} can_go_out={} can_go_home={} traverse_state={} traverse_cmd_sign={:?} traverse_raw_delta={} tension_zeroed={} laser={} tension_v={:.3} raw_norm={:.4} tension_deg={:.2} | spool vel={} acc={} c1=0x{:02X} c2=0x{:02X} c3=0x{:02X} pos={} s1=0x{:02X} s2=0x{:02X} s3=0x{:02X} ack={} di1={} | traverse vel={} acc={} c1=0x{:02X} c2=0x{:02X} c3=0x{:02X} pos={} s1=0x{:02X} s2=0x{:02X} s3=0x{:02X} ack={} ref_ack={} ref_ok={} busy={} di1={} di2={} | puller vel={} acc={} c1=0x{:02X} c2=0x{:02X} c3=0x{:02X} pos={} s1=0x{:02X} s2=0x{:02X} s3=0x{:02X} ack={} di1={}",
            self.mode,
            self.spool_mode,
            self.traverse_mode,
            self.puller_mode,
            self.can_wind(),
            self.can_go_in(),
            self.can_go_out(),
            self.can_go_home(),
            self.traverse_controller.debug_state(),
            self.traverse_controller.debug_homing_command_sign(),
            self.last_traverse_debug_raw_delta,
            self.tension_arm.zeroed,
            self.laser.get(),
            tension_voltage,
            self.tension_arm_raw.get_normalized(),
            self.tension_arm.get_angle().get::<units::angle::degree>(),
            self.spool.get_speed(),
            self.spool.get_target_acceleration(),
            self.spool.get_control_byte1(),
            self.spool.get_control_byte2(),
            self.spool.get_control_byte3(),
            self.spool.get_position(),
            self.spool.get_status_byte1(),
            self.spool.get_status_byte2(),
            self.spool.get_status_byte3(),
            self.spool.get_s1_bit3_speed_mode_ack(),
            self.spool.get_s3_bit0(),
            self.traverse.get_speed(),
            self.traverse.get_target_acceleration(),
            self.traverse.get_control_byte1(),
            self.traverse.get_control_byte2(),
            self.traverse.get_control_byte3(),
            self.traverse.get_position(),
            self.traverse.get_status_byte1(),
            self.traverse.get_status_byte2(),
            self.traverse.get_status_byte3(),
            self.traverse.get_s1_bit3_speed_mode_ack(),
            self.traverse.get_s1_bit5_reference_mode_ack(),
            self.traverse.get_s2_reference_ok(),
            self.traverse.get_s2_busy(),
            self.traverse.get_s3_bit0(),
            self.traverse.get_s3_bit1(),
            self.puller.get_speed(),
            self.puller.get_target_acceleration(),
            self.puller.get_control_byte1(),
            self.puller.get_control_byte2(),
            self.puller.get_control_byte3(),
            self.puller.get_position(),
            self.puller.get_status_byte1(),
            self.puller.get_status_byte2(),
            self.puller.get_status_byte3(),
            self.puller.get_s1_bit3_speed_mode_ack(),
            self.puller.get_s3_bit0(),
        )
    }

    pub fn emit_debug_snapshot_if_changed(&mut self) {
        let traverse_raw_position = self.traverse.get_raw_position();
        self.last_traverse_debug_raw_delta = self
            .last_traverse_debug_raw_position
            .map(|last| traverse_raw_position - last)
            .unwrap_or(0);
        self.last_traverse_debug_raw_position = Some(traverse_raw_position);

        let debug_key = self.build_debug_key();

        if self.last_debug_signature.as_ref() != Some(&debug_key) {
            let snapshot = self.build_debug_snapshot();
            tracing::info!("{}", snapshot);
            self.last_debug_signature = Some(debug_key);
        }
    }

    pub fn emit_control_loop_debug_if_due(
        &mut self,
        now: Instant,
        traverse_in_error_recovery: bool,
        traverse_should_be_energized: bool,
    ) {
        let active = self.mode != Winder2Mode::Standby
            || self.spool_mode != SpoolMode::Standby
            || self.traverse_mode != TraverseMode::Standby
            || self.puller_mode != PullerMode::Standby
            || self.traverse_controller.is_going_home()
            || self.traverse_controller.is_going_in()
            || self.traverse_controller.is_going_out()
            || self.traverse_controller.is_traversing();

        if !active
            || now.duration_since(self.last_control_loop_debug_emit) < Duration::from_millis(500)
        {
            return;
        }

        self.last_control_loop_debug_emit = now;

        tracing::info!(
            "WagoWinder loop | mode={:?} spool_mode={:?} traverse_mode={:?} puller_mode={:?} can_go_home={} can_wind={} traverse_state={} cmd_sign={:?} error_recovery={} should_energize={} enabled={} target={} actual_reg={} raw_pos={} raw_delta={} di1={} home_di2={} c1=0x{:02X} c2=0x{:02X} c3=0x{:02X} s1=0x{:02X} s2=0x{:02X} s3=0x{:02X} speed_ack={} ref_ack={} ref_ok={} busy={} | spool target={} actual_reg={} c1=0x{:02X} s1=0x{:02X} s2=0x{:02X} s3=0x{:02X} | puller target={} actual_reg={} c1=0x{:02X} s1=0x{:02X} s2=0x{:02X} s3=0x{:02X}",
            self.mode,
            self.spool_mode,
            self.traverse_mode,
            self.puller_mode,
            self.can_go_home(),
            self.can_wind(),
            self.traverse_controller.debug_state(),
            self.traverse_controller.debug_homing_command_sign(),
            traverse_in_error_recovery,
            traverse_should_be_energized,
            self.traverse.is_enabled(),
            self.traverse.get_speed(),
            self.traverse.get_actual_velocity_register(),
            self.traverse.get_raw_position(),
            self.last_traverse_debug_raw_delta,
            self.traverse.get_s3_bit0(),
            self.traverse.get_s3_bit1(),
            self.traverse.get_control_byte1(),
            self.traverse.get_control_byte2(),
            self.traverse.get_control_byte3(),
            self.traverse.get_status_byte1(),
            self.traverse.get_status_byte2(),
            self.traverse.get_status_byte3(),
            self.traverse.get_s1_bit3_speed_mode_ack(),
            self.traverse.get_s1_bit5_reference_mode_ack(),
            self.traverse.get_s2_reference_ok(),
            self.traverse.get_s2_busy(),
            self.spool.get_speed(),
            self.spool.get_actual_velocity_register(),
            self.spool.get_control_byte1(),
            self.spool.get_status_byte1(),
            self.spool.get_status_byte2(),
            self.spool.get_status_byte3(),
            self.puller.get_speed(),
            self.puller.get_actual_velocity_register(),
            self.puller.get_control_byte1(),
            self.puller.get_status_byte1(),
            self.puller.get_status_byte2(),
            self.puller.get_status_byte3(),
        );
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
                    self.spool_speed_controller.set_enabled(false);
                    self.spool_speed_controller.set_speed(AngularVelocity::ZERO);
                    let _ = self.spool.set_speed(0.0);
                }
                SpoolMode::Wind => {
                    self.spool.set_enabled(true);
                    self.spool_speed_controller.reset();
                    self.spool_speed_controller.set_enabled(true);
                }
            },
            SpoolMode::Hold => match mode {
                SpoolMode::Standby => {
                    // From [`SpoolMode::Hold`] to [`SpoolMode::Standby`]
                    self.spool_speed_controller.set_enabled(false);
                    self.spool_speed_controller.set_speed(AngularVelocity::ZERO);
                    self.spool.request_fast_stop();
                    let _ = self.spool.set_speed(0.0);
                    self.spool.set_enabled(false);
                }
                SpoolMode::Hold => {
                    self.spool_speed_controller.set_enabled(false);
                    self.spool_speed_controller.set_speed(AngularVelocity::ZERO);
                    self.spool.request_fast_stop();
                    let _ = self.spool.set_speed(0.0);
                }
                SpoolMode::Wind => {
                    // From [`SpoolMode::Hold`] to [`SpoolMode::Wind`]
                    self.spool_speed_controller.reset();
                    self.spool_speed_controller.set_enabled(true);
                }
            },
            SpoolMode::Wind => match mode {
                SpoolMode::Standby => {
                    // From [`SpoolMode::Wind`] to [`SpoolMode::Standby`]
                    self.spool_speed_controller.set_speed(AngularVelocity::ZERO);
                    self.spool.request_fast_stop();
                    let _ = self.spool.set_speed(0.0);
                    self.spool.set_enabled(false);
                    self.spool_speed_controller.set_enabled(false);
                }
                SpoolMode::Hold => {
                    // From [`SpoolMode::Wind`] to [`SpoolMode::Hold`]
                    self.spool_speed_controller.set_enabled(false);
                    self.spool_speed_controller.set_speed(AngularVelocity::ZERO);
                    self.spool.request_fast_stop();
                    let _ = self.spool.set_speed(0.0);
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
                    self.puller_speed_controller.set_enabled(false);
                    let _ = self.puller.set_speed(0.0);
                }
                PullerMode::Pull => {
                    // From [`PullerMode::Standby`] to [`PullerMode::Pull`]
                    self.puller.set_enabled(true);
                    self.puller_speed_controller.reset();
                    self.puller_speed_controller.set_enabled(true);
                }
            },
            PullerMode::Hold => match mode {
                PullerMode::Standby => {
                    // From [`PullerMode::Hold`] to [`PullerMode::Standby`]
                    self.puller_speed_controller.set_enabled(false);
                    let _ = self.puller.set_speed(0.0);
                    self.puller.set_enabled(false);
                }
                PullerMode::Hold => {
                    self.puller_speed_controller.set_enabled(false);
                    let _ = self.puller.set_speed(0.0);
                }
                PullerMode::Pull => {
                    // From [`PullerMode::Hold`] to [`PullerMode::Pull`]
                    self.puller_speed_controller.reset();
                    self.puller_speed_controller.set_enabled(true);
                }
            },
            PullerMode::Pull => match mode {
                PullerMode::Standby => {
                    // From [`PullerMode::Pull`] to [`PullerMode::Standby`]
                    let _ = self.puller.set_speed(0.0);
                    self.puller.set_enabled(false);
                    self.puller_speed_controller.set_enabled(false);
                }
                PullerMode::Hold => {
                    // From [`PullerMode::Pull`] to [`PullerMode::Hold`]
                    self.puller_speed_controller.set_enabled(false);
                    let _ = self.puller.set_speed(0.0);
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
        let meters_pulled_this_interval =
            Length::new::<meter>(self.actual_puller_linear_speed().get::<meter_per_second>() * dt);

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
}

#[cfg(feature = "mock-machine")]
pub use mock::WagoWinder;

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

#[cfg(not(feature = "mock-machine"))]
impl std::fmt::Display for WagoWinder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WagoWinder")
    }
}

#[cfg(not(feature = "mock-machine"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_traverse_limits() {
        // Test case 1: Valid limits with exactly 1.0mm difference (should pass)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(16.0);
        assert!(WagoWinder::validate_traverse_limits(inner, outer));

        // Test case 2: Invalid limits with exactly 0.9mm difference (should fail)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.9);
        assert!(!WagoWinder::validate_traverse_limits(inner, outer));

        // Test case 3: Invalid limits with less than 0.9mm difference (should fail)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.5);
        assert!(!WagoWinder::validate_traverse_limits(inner, outer));

        // Test case 4: Invalid limits where inner equals outer (should fail)
        let inner = Length::new::<millimeter>(20.0);
        let outer = Length::new::<millimeter>(20.0);
        assert!(!WagoWinder::validate_traverse_limits(inner, outer));

        // Test case 5: Invalid limits where inner is greater than outer (should fail)
        let inner = Length::new::<millimeter>(25.0);
        let outer = Length::new::<millimeter>(20.0);
        assert!(!WagoWinder::validate_traverse_limits(inner, outer));

        // Test case 6: Valid limits with large difference (should pass)
        let inner = Length::new::<millimeter>(10.0);
        let outer = Length::new::<millimeter>(80.0);
        assert!(WagoWinder::validate_traverse_limits(inner, outer));

        // Test case 7: Edge case - exactly 0.91mm difference (should pass)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.91);
        assert!(WagoWinder::validate_traverse_limits(inner, outer));

        // Test case 8: Edge case - exactly 0.89mm difference (should fail)
        let inner = Length::new::<millimeter>(15.0);
        let outer = Length::new::<millimeter>(15.89);
        assert!(!WagoWinder::validate_traverse_limits(inner, outer));
    }
}
