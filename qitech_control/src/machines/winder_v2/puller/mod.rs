use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use qitech_framework::machine::ActErrorKind;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::Acceleration;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Jerk;
use qitech_lib::units::Length;
use qitech_lib::units::Velocity;
use qitech_lib::units::acceleration::meter_per_minute_per_second;
use qitech_lib::units::jerk::meter_per_minute_per_second_squared;
use qitech_lib::units::length::centimeter;
use qitech_lib::units::velocity::meter_per_minute;

use crate::controllers::second_degree_motion::LinearJerkSpeedControllerDT;
use crate::converters::LinearStepConverter;
use crate::machines::winder_v2::types::LaserSubscription;
use crate::types::RotationDirection;

mod types;
use types::GearRatio;
use types::Mode;
use types::SpeedControlAlgorithm;

mod speed_controller;
use speed_controller::SpeedController;

pub struct Puller {
    // --- hardware ---
    device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,

    // --- config properties ---
    direction: ConfigProperty<RotationDirection>,
    gear_ratio: ConfigProperty<GearRatio>,

    // --- state properties ---
    mode: StateProperty<Mode>,

    // --- measurements ---
    speed: Measurement<Velocity>,

    /// Computes the target linear speed.
    speed_controller: SpeedController,

    /// Applies acceleration and jerk limits to the target speed.
    acceleration_controller: LinearJerkSpeedControllerDT,

    /// Converts the limited linear speed to angular speed.
    converter: LinearStepConverter,
}

// --- init ---
impl Puller {
    pub fn init<const VARIANT: usize>(
        ctx: &mut BuildContext,
        device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    ) -> BuildResult<Self> {
        let speed = Velocity::new::<meter_per_minute>(50.0);
        let acceleration = Acceleration::new::<meter_per_minute_per_second>(5.0);
        let jerk = Jerk::new::<meter_per_minute_per_second_squared>(10.0);

        let acceleration_controller =
            LinearJerkSpeedControllerDT::new_simple(Some(speed), acceleration, jerk);

        let converter = LinearStepConverter::from_diameter(
            // Assuming 200 steps per revolution for the puller stepper,
            200,
            // 8cm diameter of the puller wheel
            Length::new::<centimeter>(8.0),
        );

        let instance = Self {
            device,
            direction: ctx
                .config::<RotationDirection>("puller.direction")
                .build()?,
            gear_ratio: ctx.config::<GearRatio>("puller.gear_ratio").build()?,
            mode: ctx.state::<Mode>("puller.mode").build()?,
            speed: ctx
                .measurement::<meter_per_minute>("puller.speed")
                .build()?,
            speed_controller: SpeedController::new::<VARIANT>(ctx)?,
            acceleration_controller,
            converter,
        };

        Ok(instance)
    }
}

// --- public api ---
impl Puller {
    pub fn set_mode(&mut self, mode: Mode) {
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
        dt: Duration,
        laser_subscription: Option<&LaserSubscription>,
    ) -> Result<(), ActErrorKind> {
        self.update_speed(dt, laser_subscription);
        self.update_device()?;
        Ok(())
    }

    fn update_speed(&mut self, dt: Duration, laser_subscription: Option<&LaserSubscription>) {
        // --- update the speed controller ---
        self.speed_controller
            .update(dt, self.speed.get(), laser_subscription);

        // --- only use the target speed if we are actually pulling ---
        let speed_target = match self.mode.get() {
            Mode::Pull => {
                let mut speed_target = self.speed_controller.speed_target();
                speed_target *= self.gear_ratio.get().multiplier();
                speed_target *= self.direction.get().modifier();
                speed_target
            }

            _ => Velocity::ZERO,
        };

        // --- apply acceleration to target speed to get output speed ---
        let speed = self.acceleration_controller.update(dt, speed_target);

        // --- apply ---
        self.speed.set(speed);
    }

    pub fn speed(&self) -> Velocity {
        self.speed.get()
    }

    pub fn angular_velocity(&self) -> AngularVelocity {
        self.converter
            .velocity_to_angular_velocity(self.speed.get())
    }
}

// --- internals ---
impl Puller {
    const PORT: usize = 0;
    fn update_device(&mut self) -> Result<(), ActErrorKind> {
        let angular_velocity = self.angular_velocity();
        let steps_per_second = self.converter.angular_velocity_to_steps(angular_velocity);

        self.device
            .borrow_mut()
            .set_speed(Self::PORT, steps_per_second)
            .map_err(|e| ActErrorKind::Custom(e.to_string()))
    }
}
