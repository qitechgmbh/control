use std::time::Instant;

use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_lib::units::Angle;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::Velocity;

use crate::machines::winder_v2::build::TensionArm;
use crate::machines::winder_v2::puller::Puller;
use crate::machines::winder_v2::spool::SpeedAlgorithmAdaptive;
use crate::machines::winder_v2::spool::SpeedAlgorithmMinMax;
use crate::machines::winder_v2::spool::SpeedControlAlgorithm;
use crate::machines::winder_v2::utils::Clamping;
use crate::machines::winder_v2::utils::FilamentTensionCalculator;
use crate::machines::winder_v2::utils::clamp_revolution_uom;

pub struct SpeedController {
    // --- config ---
    pub(crate) speed_min: ConfigProperty<AngularVelocity>,
    pub(crate) speed_max: ConfigProperty<AngularVelocity>,

    // --- state ---
    pub(crate) enabled: StateProperty<bool>,

    // --- measurements ---
    pub(crate) speed: Measurement<AngularVelocity>,

    // --- algorithms ----
    pub(crate) selected: SpeedControlAlgorithm,
    pub(crate) min_max: SpeedAlgorithmMinMax,
    pub(crate) adaptive: SpeedAlgorithmAdaptive,

    // --- misc ---
    pub(crate) filament_tension_calculator: FilamentTensionCalculator,
    pub(crate) filament_tension: Measurement<Option<f64>>,
}

impl SpeedController {
    /// Absolute safety limit (in RPM) that the spool speed
    /// can never exceed to protect hardware
    const SPEED_MAX_SAFETY: f64 = 600.0;

    pub fn update(&mut self, now: Instant, puller: &Puller, tension_arm: &TensionArm) {
        self.update_filament_tension(tension_arm);

        let input = SpeedAlgorithmInput {
            now,
            enabled: self.enabled.get(),
            speed_min: self.speed_min.get(),
            speed_max: self.speed_max.get(),
            puller_speed: puller.speed(),
            tension_arm_angle: tension_arm.angle(),
            filament_tension: self.filament_tension.get(),
        };

        let speed = match self.selected {
            SpeedControlAlgorithm::Adaptive => self.adaptive.compute(input),
            SpeedControlAlgorithm::MinMax => self.min_max.compute(input),
        };

        let speed_clamped = speed.max(self.speed_min.get()).min(self.speed_max.get());
        self.speed.set(speed_clamped);
    }

    fn update_filament_tension(&mut self, tension_arm: &TensionArm) {
        let tension_arm_angle = tension_arm.angle();

        let (tension_arm_revolution, clamping) = clamp_revolution_uom(
            tension_arm_angle,
            // inverted because min angle is max tension
            self.filament_tension_calculator.get_max_angle(),
            self.filament_tension_calculator.get_min_angle(),
        );

        self.filament_tension
            .set(if !matches!(clamping, Clamping::None) {
                Some(
                    self.filament_tension_calculator
                        .calc_filament_tension(tension_arm_revolution),
                )
            } else {
                None
            });
    }
}

pub struct SpeedAlgorithmInput {
    pub now: Instant,
    pub enabled: bool,
    pub speed_min: AngularVelocity,
    pub speed_max: AngularVelocity,
    pub puller_speed: Velocity,
    pub tension_arm_angle: Angle,
    pub filament_tension: Option<f64>,
}
