pub mod act;
pub mod api;
pub mod emit;
pub mod new;
pub mod rewind_control;

use crate::winder2::{
    puller_speed_controller::PullerSpeedController, spool_speed_controller::SpoolSpeedController,
    tension_arm::TensionArm, traverse_controller::TraverseController,
};
use crate::{MachineMessage, QiTechMachine, MACHINE_REWINDER_V1, VENDOR_QITECH};
use api::RewinderNamespace;
use control_core::converters::angular_step_converter::AngularStepConverter;
#[cfg(not(feature = "mock-machine"))]
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::{
    ethercat_hal::io::digital_output::DigitalOutputDevice,
    machines::{MachineIdentification, MachineIdentificationUnique},
    units::{length::millimeter, Length, Velocity},
};
use rewind_control::RewindControlState;
use std::{cell::RefCell, rc::Rc, time::Instant};
#[cfg(not(feature = "mock-machine"))]
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub const TRAVERSE_PORT: usize = 0;
pub const PULLER_PORT: usize = 0;
pub const TAKEUP_SPOOL_PORT: usize = 0;
pub const SOURCE_SPOOL_PORT: usize = 0;

pub const EK1100_ROLE: u16 = 0;
pub const EL2002_ROLE: u16 = 1;
pub const TAKEUP_SPOOL_ROLE: u16 = 2;
pub const TRAVERSE_ROLE: u16 = 3;
pub const PULLER_ROLE: u16 = 4;
pub const SOURCE_SPOOL_ROLE: u16 = 5;

impl QiTechMachine for Rewinder {}

pub struct Rewinder {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,

    pub digital_outputs: Rc<RefCell<dyn DigitalOutputDevice>>,
    pub traverse: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    pub takeup_spool: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    pub puller: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    pub source_spool: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,

    pub takeup_tension_arm: TensionArm,
    pub source_tension_arm: TensionArm,

    namespace: RewinderNamespace,
    last_measurement_emit: Instant,
    last_rewind_diagnostics_log: Instant,
    pub machine_identification_unique: MachineIdentificationUnique,

    pub mode: RewinderMode,
    pub puller_speed_controller: PullerSpeedController,
    pub takeup_spool_speed_controller: SpoolSpeedController,
    pub source_spool_speed_controller: SpoolSpeedController,
    pub takeup_spool_step_converter: AngularStepConverter,
    pub source_spool_step_converter: AngularStepConverter,
    pub traverse_controller: TraverseController,
    pub rewind_phase: RewindPhase,
    rewind_hard_stop_reason: Option<String>,
    rewind_puller_command_speed: Option<Velocity>,
    pub rewind_control: RewindControlState,
    emitted_default_state: bool,
    last_can_rewind: bool,
}

impl Rewinder {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_REWINDER_V1,
    };

    pub fn can_rewind(&self) -> bool {
        self.rewind_block_reason().is_none()
    }

    pub fn rewind_block_reason(&self) -> Option<&'static str> {
        self.rewind_block_reason_with_start_window(true)
    }

    pub fn runtime_rewind_block_reason(&self) -> Option<&'static str> {
        self.rewind_block_reason_with_start_window(false)
    }

    fn rewind_block_reason_with_start_window(
        &self,
        enforce_start_window: bool,
    ) -> Option<&'static str> {
        if !self.takeup_tension_arm.zeroed {
            Some("takeup tension arm is not zeroed")
        } else if !self.source_tension_arm.zeroed {
            Some("source tension arm is not zeroed")
        } else if !self.traverse_controller.is_homed() {
            Some("traverse is not homed")
        } else if self.traverse_controller.is_going_home() {
            Some("traverse is still homing")
        } else if enforce_start_window
            && self
                .source_tension_arm
                .get_angle()
                .map(Self::normalize_tension_arm_angle_deg)
                .map_or(true, |angle| {
                    !(Self::SOURCE_TENSION_ARM_START_MIN_ANGLE_DEG
                        ..=Self::SOURCE_TENSION_ARM_START_MAX_ANGLE_DEG)
                        .contains(&angle)
                })
        {
            Some("source tension arm is outside rewind start range")
        } else if enforce_start_window
            && self
                .takeup_tension_arm
                .get_angle()
                .map(Self::normalize_tension_arm_angle_deg)
                .map_or(true, |angle| {
                    !(Self::TAKEUP_TENSION_ARM_START_MIN_ANGLE_DEG
                        ..=Self::TAKEUP_TENSION_ARM_START_MAX_ANGLE_DEG)
                        .contains(&angle)
                })
        {
            Some("takeup tension arm is outside rewind start range")
        } else if self
            .source_tension_arm
            .get_angle()
            .map(Self::normalize_tension_arm_angle_deg)
            .map_or(true, |angle| {
                !(Self::SOURCE_TENSION_ARM_MIN_ANGLE_DEG..=Self::SOURCE_TENSION_ARM_MAX_ANGLE_DEG)
                    .contains(&angle)
            })
        {
            Some("source tension arm is outside rewind range")
        } else if self
            .takeup_tension_arm
            .get_angle()
            .map(Self::normalize_tension_arm_angle_deg)
            .map_or(true, |angle| {
                !(Self::TAKEUP_TENSION_ARM_MIN_ANGLE_DEG..=Self::TAKEUP_TENSION_ARM_MAX_ANGLE_DEG)
                    .contains(&angle)
            })
        {
            Some("takeup tension arm is outside rewind range")
        } else {
            None
        }
    }

    pub fn rewind_motion_permitted(&self) -> bool {
        matches!(self.mode, RewinderMode::Rewind)
    }

    pub fn puller_motion_permitted(&self) -> bool {
        matches!(self.mode, RewinderMode::Pull)
            || (self.rewind_motion_permitted()
                && matches!(
                    self.rewind_phase,
                    RewindPhase::CrawlStart | RewindPhase::Rewind
                ))
    }

    pub fn takeup_spool_motion_permitted(&self) -> bool {
        self.rewind_motion_permitted()
            && matches!(
                self.rewind_phase,
                RewindPhase::Precharge | RewindPhase::CrawlStart | RewindPhase::Rewind
            )
    }

    pub fn source_spool_motion_permitted(&self) -> bool {
        self.rewind_motion_permitted()
            && matches!(
                self.rewind_phase,
                RewindPhase::Precharge | RewindPhase::CrawlStart | RewindPhase::Rewind
            )
    }

    pub fn traverse_motion_permitted(&self) -> bool {
        matches!(self.mode, RewinderMode::Hold)
            || (self.rewind_motion_permitted()
                && matches!(
                    self.rewind_phase,
                    RewindPhase::CrawlStart | RewindPhase::Rewind
                ))
    }

    fn validate_traverse_limits(inner: Length, outer: Length) -> bool {
        outer > inner + Length::new::<millimeter>(0.9)
    }

    pub fn sync_traverse_speed(&mut self) {
        let traverse = &mut *self.traverse.borrow_mut();
        if self.traverse_motion_permitted() {
            self.traverse_controller
                .update_speed(traverse, self.takeup_spool_speed_controller.get_speed());
        } else {
            let _ = traverse.set_speed(TRAVERSE_PORT, 0.0);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RewinderMode {
    Standby,
    Hold,
    Pull,
    Rewind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewindPhase {
    Idle,
    Validate,
    Precharge,
    CrawlStart,
    Rewind,
    FaultHold,
}
