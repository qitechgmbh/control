mod act;
mod adaptive_spool_speed_controller;
mod api;
mod clamp_revolution;
mod filament_tension;
mod minmax_spool_speed_controller;
mod new;
mod puller_speed_controller;
mod spool_speed_controller;
mod tension_arm;
mod traverse_controller;

use crate::converters::angular_step_converter::AngularStepConverter;
use crate::machines::winder_v2::api::Measurements;
use crate::machines::winder_v2::new::TensionArm;
use crate::machines::winder_v2::puller_speed_controller::PullerSpeedController;
use crate::machines::winder_v2::spool_speed_controller::SpoolSpeedController;
use crate::machines::winder_v2::traverse_controller::Traverse;
use crate::machines::winder_v2::types::LaserSubscription;
use crate::machines::winder_v2::types::Mode;
use crate::machines::winder_v2::types::PullerMode;
use crate::machines::winder_v2::types::SpoolAutomaticActionMode;
use crate::machines::winder_v2::types::SpoolMode;
use crate::machines::winder_v2::types::TraverseMode;
use qitech_framework::MachineIdentification;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::MachineDescriptor;
use qitech_framework::machine::OperationCapability;
use qitech_framework::machine::StateProperty;
use qitech_framework::vendors;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
#[cfg(not(feature = "mock-machine"))]
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::ConstZero;
use qitech_lib::units::{Length, length::meter, velocity::meter_per_second};
use std::time::Instant;
use std::{cell::RefCell, rc::Rc};

mod types;

pub const TRAVERSE_PORT: usize = 0;
pub const LASER_PORT: usize = 0;
pub const PULLER_PORT: usize = 0;
pub const SPOOL_PORT: usize = 0;
pub const TRAVERSE_END_STOP_PORT: usize = 0;

pub const VARIANT_REGULAR: usize = 0;
pub const VARIANT_7031_SPOOL: usize = 1;

pub struct SpoolAutomaticAction {
    pub progress: Length,
    progress_last_check: Instant,
    pub target_length: ConfigProperty<Length>,
    pub mode: ConfigProperty<SpoolAutomaticActionMode>,
}

#[allow(non_camel_case_types)]
pub type WinderV1_Regular = WinderV1<VARIANT_REGULAR>;

#[allow(non_camel_case_types)]
pub type WinderV1_7031_Spool = WinderV1<VARIANT_7031_SPOOL>;

pub struct WinderV1<const VARIANT: usize> {
    // drivers
    pub traverse: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    pub puller: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    pub spool: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    pub tension_arm: TensionArm,

    pub laser: Rc<RefCell<dyn DigitalOutputDevice>>,
    pub traverse_controller: Traverse,

    // mode
    pub mode: StateProperty<Mode>,
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

    // --- resource api migration ---
    laser_enabled: StateProperty<bool>,
    measurements: Measurements,

    // --- subscriptions ---
    pub laser_subscription: Option<LaserSubscription>,
}

impl MachineDescriptor for WinderV1<VARIANT_REGULAR> {
    const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: vendors::QITECH.id,
        machine_id: 2,
    };

    const SCHEMA: &'static str = include_str!("../../../schemas/winder_v1.yaml");
}

impl MachineDescriptor for WinderV1<VARIANT_7031_SPOOL> {
    const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: vendors::QITECH.id,
        machine_id: 98,
    };

    const SCHEMA: &'static str = include_str!("../../../schemas/winder_v1_7031_0030_spool.yaml");
}

impl<const VARIANT: usize> WinderV1<VARIANT> {
    pub fn sync_traverse_speed(&mut self) {
        let traverse = &mut *self.traverse.borrow_mut();
        self.traverse_controller
            .update_speed(traverse, self.spool_speed_controller.get_speed());
    }

    /// Can go to inner limit capability check
    pub fn traverse_can_goto_limit_inner(&self) -> OperationCapability {
        if !self.traverse_controller.is_homed() {
            return OperationCapability::Forbidden {
                reason: "traverse is not homed".to_string(),
            };
        }

        if self.traverse_mode == TraverseMode::Standby {
            return OperationCapability::Forbidden {
                reason: "traverse is in standby".to_string(),
            };
        }

        if self.traverse_controller.is_going_in() {
            return OperationCapability::Forbidden {
                reason: "traverse is already going in".to_string(),
            };
        }

        if self.traverse_controller.is_going_home() {
            return OperationCapability::Forbidden {
                reason: "traverse is currently homing".to_string(),
            };
        }

        if self.traverse_controller.is_traversing() {
            return OperationCapability::Forbidden {
                reason: "traverse is currently traversing".to_string(),
            };
        }

        if self.mode.get() == Mode::Wind {
            return OperationCapability::Forbidden {
                reason: "winder is in wind mode".to_string(),
            };
        }

        OperationCapability::Allowed
    }

    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_spool_mode(&mut self, mode: Mode) {
        // Convert to `Winder2Mode` to `SpoolMode`
        let mode: SpoolMode = mode.into();
        let spool = &mut *self.spool.borrow_mut();
        // Transition matrix
        match self.spool_mode {
            SpoolMode::Standby => match mode {
                SpoolMode::Standby => {}
                SpoolMode::Hold => {
                    // From [`SpoolMode::Standby`] to [`SpoolMode::Hold`]
                    spool.set_enabled(SPOOL_PORT, true);
                }
                SpoolMode::Wind => {
                    spool.set_enabled(SPOOL_PORT, true);
                    // self.spool_speed_controller.reset();
                    self.spool_speed_controller.set_enabled(true);
                }
            },
            SpoolMode::Hold => match mode {
                SpoolMode::Standby => {
                    // From [`SpoolMode::Hold`] to [`SpoolMode::Standby`]
                    spool.set_enabled(SPOOL_PORT, false);
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
                    spool.set_enabled(SPOOL_PORT, false);
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
    fn set_puller_mode(&mut self, mode: Mode) {
        // Convert to `Winder2Mode` to `PullerMode`
        let mode: PullerMode = mode.into();
        let puller = &mut *self.puller.borrow_mut();

        // Transition matrix
        match self.puller_mode {
            PullerMode::Standby => match mode {
                PullerMode::Standby => {}
                PullerMode::Hold => {
                    // From [`PullerMode::Standby`] to [`PullerMode::Hold`]
                    puller.set_enabled(PULLER_PORT, true);
                }
                PullerMode::Pull => {
                    // From [`PullerMode::Standby`] to [`PullerMode::Pull`]
                    puller.set_enabled(PULLER_PORT, true);
                    self.puller_speed_controller.set_enabled(true);
                }
            },
            PullerMode::Hold => match mode {
                PullerMode::Standby => {
                    // From [`PullerMode::Hold`] to [`PullerMode::Standby`]
                    puller.set_enabled(PULLER_PORT, false);
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
                    puller.set_enabled(PULLER_PORT, false);
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
        let dt = now
            .duration_since(self.spool_automatic_action.progress_last_check)
            .as_secs_f64();

        let meters_pulled_this_interval = Length::new::<meter>(
            self.puller_speed_controller
                .last_speed
                .get::<meter_per_second>()
                * dt,
        );

        self.spool_automatic_action.progress += meters_pulled_this_interval.abs();
        self.spool_automatic_action.progress_last_check = now;
    }

    pub fn sync_puller_speed(&mut self, t: Instant) {
        let angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
        let steps_per_second = self
            .puller_speed_controller
            .converter
            .angular_velocity_to_steps(angular_velocity);
        let puller = &mut *self.puller.borrow_mut();
        let _ = puller.set_speed(PULLER_PORT, steps_per_second);
    }
}
