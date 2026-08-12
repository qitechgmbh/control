use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use qitech_framework::EnumProperty;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Length;

use crate::converters::AngularStepConverter;
use crate::machines::winder_v2::build::TensionArm;
use crate::machines::winder_v2::puller::Puller;
use crate::types::RotationDirection;

mod types;
pub use types::Mode;

mod speed_controller;
use speed_controller::SpeedController;

mod speed_controller_adaptive;
pub(super) use speed_controller_adaptive::SpeedAlgorithmAdaptive;

mod speed_algorithm_min_max;
pub(super) use speed_algorithm_min_max::SpeedAlgorithmMinMax;

#[derive(Debug, Default, Clone, Copy, PartialEq, EnumProperty)]
enum SpeedControlAlgorithm {
    #[default]
    Adaptive,
    MinMax,
}

pub struct Spool {
    pub(crate) device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,

    // --- config properties ---
    pub(crate) direction: ConfigProperty<RotationDirection>,
    pub(crate) regulation_mode: ConfigProperty<SpeedControlAlgorithm>,

    // --- state properties ---
    pub(super) mode: StateProperty<Mode>,

    // --- measurements ---
    pub(crate) velocity: Measurement<AngularVelocity>,
    pub(crate) progress: Measurement<Length>,

    // --- speed controllers ---
    pub(crate) speed_controller: SpeedController,

    // --- converters ---
    pub(crate) step_converter: AngularStepConverter,
}

impl Spool {
    const PORT: usize = 0;

    pub fn apply_mode(&mut self, mode: Mode) {
        if self.mode.set(mode) {
            let enabled = match self.mode.get() {
                Mode::Standby => false,
                Mode::Hold | Mode::Wind => true,
            };

            self.device.borrow_mut().set_enabled(Self::PORT, enabled);
        }
    }

    pub fn update(&mut self, now: Instant, puller: &Puller, tension_arm: &TensionArm) {
        self.speed_controller.update(now, puller, tension_arm);
        self.sync();
    }

    fn sync(&mut self) {
        let angular_velocity = self.speed_controller.speed.get() * self.direction.get().modifier();

        let steps_per_second = self
            .step_converter
            .angular_velocity_to_steps(angular_velocity);

        self.device
            .borrow_mut()
            .set_speed(Self::PORT, steps_per_second);
    }

    pub fn direction(&self) -> RotationDirection {
        self.direction.get()
    }

    pub fn reset_progress(&mut self) {
        self.progress.set(Length::ZERO);
    }
}
