use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use qitech_control_core::converters::angular_step_converter::AngularStepConverter;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Length;
use qitech_lib::units::angular_velocity::revolution_per_minute;
use qitech_lib::units::length::meter;
use qitech_lib::units::velocity::meter_per_second;

use crate::machines::winder_v2::TensionArm;
use crate::machines::winder_v2::WinderV1;
use crate::machines::winder_v2::puller::Puller;
use crate::types::RotationDirection;

mod types;
pub use types::Mode;
pub use types::SpeedControlAlgorithm;

mod speed_controller;
use speed_controller::SpeedController;

mod speed_algorithm_adaptive;
pub(super) use speed_algorithm_adaptive::SpeedAlgorithmAdaptive;

mod speed_algorithm_min_max;
pub(super) use speed_algorithm_min_max::SpeedAlgorithmMinMax;

pub struct Spool {
    // --- hardware ---
    pub(crate) device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,

    // --- config ---
    pub(crate) direction: ConfigProperty<RotationDirection>,

    // --- state ---
    pub(super) mode: StateProperty<Mode>,

    // --- measurements ---
    pub(crate) velocity: Measurement<AngularVelocity>,
    pub(crate) progress: Measurement<Length>,

    // --- controllers ---
    pub(crate) speed_controller: SpeedController,

    // --- converters ---
    pub(crate) step_converter: AngularStepConverter,
}

// --- constants ---
impl Spool {
    const PORT: usize = 0;
}

// --- init ---
impl Spool {
    pub fn init<const VARIANT: usize>(
        ctx: &mut BuildContext,
        device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    ) -> BuildResult<Self> {
        ctx.command("spool.reset_progress")
            .execute(|m: &mut WinderV1<VARIANT>| {
                m.spool.reset_progress();
                Ok(())
            })
            .build()?;

        let speed_controller = SpeedController::init::<VARIANT>(ctx)?;

        Ok(Self {
            device,
            direction: ctx.config::<RotationDirection>("spool.direction").build()?,
            mode: ctx.state::<Mode>("spool.mode").build()?,
            velocity: ctx
                .measurement::<revolution_per_minute>("spool.rpm")
                .build()?,
            progress: ctx.measurement::<meter>("spool.progress").build()?,
            speed_controller,
            step_converter: AngularStepConverter::new(200),
        })
    }
}

// --- public interface ---
impl Spool {
    pub fn set_mode(&mut self, mode: Mode) {
        if self.mode.set(mode) {
            let enabled = match self.mode.get() {
                Mode::Standby => false,
                Mode::Hold | Mode::Wind => true,
            };

            self.device.borrow_mut().set_enabled(Self::PORT, enabled);
            self.speed_controller.set_enalbed(enabled);
        }
    }

    pub fn update(&mut self, dt: Duration, puller: &Puller, tension_arm: &TensionArm) {
        self.speed_controller.update(dt, puller, tension_arm);
        self.update_progress(dt, puller);
        self.sync();
    }

    fn update_progress(&mut self, dt: Duration, puller: &Puller) {
        let dt_seconds = dt.as_secs_f64();
        let speed_mps = puller.speed().get::<meter_per_second>();
        let distance = Length::new::<meter>(speed_mps * dt_seconds);
        self.progress.set(self.progress.get() + distance);
    }

    fn sync(&mut self) {
        let angular_velocity = self.speed_controller.speed() * self.direction.get().modifier();

        let steps_per_second = self
            .step_converter
            .angular_velocity_to_steps(angular_velocity);

        _ = self
            .device
            .borrow_mut()
            .set_speed(Self::PORT, steps_per_second);

        self.velocity.set(angular_velocity.abs());
    }

    pub fn reset_progress(&mut self) {
        self.progress.set(Length::ZERO);
    }
}
