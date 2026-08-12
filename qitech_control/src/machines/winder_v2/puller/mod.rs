use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use qitech_framework::machine::ActErrorKind;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_framework::units::length::millimeter;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Velocity;

use crate::controllers::LinearJerkSpeedController;
use crate::converters::LinearStepConverter;
use crate::machines::winder_v2::types::LaserSubscription;
use crate::types::RotationDirection;

mod types;
pub(super) use types::GearRatio;
pub(super) use types::Mode;
pub(super) use types::SpeedRegulationMode;

mod speed_algorithms;
pub(super) use speed_algorithms::SpeedAlgorithmAdaptive;

pub struct Puller {
    pub(super) device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,

    // --- config properties ---
    pub(super) direction: ConfigProperty<RotationDirection>,
    pub(super) gear_ratio: ConfigProperty<GearRatio>,
    pub(super) regulation_mode: ConfigProperty<SpeedRegulationMode>,
    pub(super) speed_target: ConfigProperty<Velocity>,

    // --- state properties ---
    pub(super) mode: StateProperty<Mode>,

    // --- measurements ---
    pub(super) speed: Measurement<Velocity>,

    /// Linear acceleration controller to dampen speed change
    pub(super) acceleration_controller: LinearJerkSpeedController,

    /// Converter for linear to angular transformations
    pub(super) converter: LinearStepConverter,

    // --- speed algorithms ---
    pub(super) sa_adaptive: SpeedAlgorithmAdaptive,
}

// --- public api ---
impl Puller {
    pub fn apply_mode(&mut self, mode: Mode) {
        if self.mode.set(mode) {
            let enabled = match self.mode.get() {
                Mode::Standby => false,
                Mode::Hold | Mode::Pull => true,
            };

            self.device.borrow_mut().set_enabled(Self::PORT, enabled);
        }
    }

    pub fn update(
        &mut self,
        now: Instant,
        laser_subscription: &Option<LaserSubscription>,
    ) -> Result<(), ActErrorKind> {
        if let Some(laser) = laser_subscription {
            self.sa_adaptive.update(
                now,
                self.speed.get(),
                laser.diameter.get_as::<millimeter>(),
                laser.diameter_target.get_as::<millimeter>(),
                laser.tolerance_lower.get_as::<millimeter>(),
                laser.tolerance_upper.get_as::<millimeter>(),
            );
        }

        self.update_speed(now);
        self.update_device()?;
        Ok(())
    }

    pub fn speed(&self) -> Velocity {
        self.speed.get()
    }

    pub fn angular_velocity(&self) -> AngularVelocity {
        self.converter
            .velocity_to_angular_velocity(self.speed.get())
    }

    pub fn on_regulation_mode_changed(&mut self) {
        if self.regulation_mode.get() != SpeedRegulationMode::Diameter {
            return;
        }

        self.sa_adaptive.reset_modulation();
    }
}

// --- internals ---
impl Puller {
    const PORT: usize = 0;

    fn update_speed(&mut self, now: Instant) {
        let speed_target = self.speed_target.get();

        let mut speed = match self.mode.get() {
            Mode::Pull => match self.regulation_mode.get() {
                SpeedRegulationMode::Direct => speed_target,
                SpeedRegulationMode::Diameter => self.sa_adaptive.compute(speed_target),
            },

            _ => Velocity::ZERO,
        };

        speed *= self.gear_ratio.get().multiplier();
        speed *= self.direction.get().modifier();
        speed = self.acceleration_controller.update(speed, now);
        self.speed.set(speed);
    }

    fn update_device(&mut self) -> Result<(), ActErrorKind> {
        let angular_velocity = self.angular_velocity();

        let steps_per_second = self.converter.angular_velocity_to_steps(angular_velocity);

        self.device
            .borrow_mut()
            .set_speed(Self::PORT, steps_per_second)
            .map_err(|e| ActErrorKind::Custom(e.to_string()))
    }
}
