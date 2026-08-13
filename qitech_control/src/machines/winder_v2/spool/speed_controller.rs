use std::time::Duration;

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_lib::units::Angle;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::Velocity;
use qitech_lib::units::angle::degree;
use qitech_lib::units::angular_velocity::revolution_per_minute;

use crate::machines::winder_v2::TensionArm;
use crate::machines::winder_v2::puller::Puller;
use crate::machines::winder_v2::spool::SpeedAlgorithmAdaptive;
use crate::machines::winder_v2::spool::SpeedAlgorithmMinMax;
use crate::machines::winder_v2::spool::SpeedControlAlgorithm;
use crate::machines::winder_v2::utils::FilamentTensionCalculator;

pub struct SpeedController {
    // --- config ---
    speed_min: ConfigProperty<AngularVelocity>,
    speed_max: ConfigProperty<AngularVelocity>,
    algorithm: ConfigProperty<SpeedControlAlgorithm>,

    // --- state ---
    enabled: StateProperty<bool>,

    // --- measurements ---
    speed_target: Measurement<AngularVelocity>,
    filament_tension: Measurement<Option<f64>>,

    // --- algorithms ----
    sa_min_max: SpeedAlgorithmMinMax,
    sa_adaptive: SpeedAlgorithmAdaptive,

    // --- calculators ---
    tension_calculator: FilamentTensionCalculator,
}

// --- constants ---
impl SpeedController {
    /// Minimum tension arm angle.
    fn tension_arm_angle_min() -> Angle {
        Angle::new::<degree>(20.0)
    }

    /// Maximum tension arm angle.
    fn tension_arm_angle_max() -> Angle {
        Angle::new::<degree>(90.0)
    }
}

// --- init ---
impl SpeedController {
    pub fn init<const VARIANT: usize>(ctx: &mut BuildContext) -> BuildResult<Self> {
        let tension_calculator = FilamentTensionCalculator::new(
            Self::tension_arm_angle_max(),
            Self::tension_arm_angle_min(),
        );

        let speed_min = ctx
            .config::<revolution_per_minute>("spool.speed_controller.speed_min")
            .default(0.0)
            .build()?;

        let speed_max = ctx
            .config::<revolution_per_minute>("spool.speed_controller.speed_max")
            .default(150.0)
            .maximum(600.0)
            .build()?;

        let algorithm = ctx
            .config::<SpeedControlAlgorithm>("spool.speed_controller.algorithm")
            .build()?;

        let enabled = ctx
            .state::<bool>("spool.speed_controller.enabled")
            .build()?;

        let speed = ctx
            .measurement::<revolution_per_minute>("spool.speed_controller.speed")
            .build()?;

        let filament_tension = ctx
            .measurement::<Option<f64>>("spool.speed_controller.filament_tension")
            .build()?;

        let sa_min_max = SpeedAlgorithmMinMax::init();
        let sa_adaptive = SpeedAlgorithmAdaptive::init(ctx)?;

        Ok(Self {
            speed_min,
            speed_max,
            algorithm,
            enabled,
            speed_target: speed,
            filament_tension,
            sa_min_max,
            sa_adaptive,
            tension_calculator,
        })
    }
}

// --- public interface ---
impl SpeedController {
    pub fn update(&mut self, dt: Duration, puller: &Puller, tension_arm: &TensionArm) {
        self.update_filament_tension(tension_arm);

        let input = SpeedAlgorithmInput {
            dt,
            enabled: self.enabled.get(),
            speed_min: self.speed_min.get(),
            speed_max: self.speed_max.get(),
            puller_speed: puller.speed(),
            filament_tension: self.filament_tension.get(),
        };

        let speed = match self.algorithm.get() {
            SpeedControlAlgorithm::Adaptive => self.sa_adaptive.compute(input),
            SpeedControlAlgorithm::MinMax => self.sa_min_max.compute(input),
        };

        let speed_clamped = speed.max(self.speed_min.get()).min(self.speed_max.get());
        self.speed_target.set(speed_clamped);
    }

    pub fn speed(&self) -> AngularVelocity {
        self.speed_target.get()
    }

    fn update_filament_tension(&mut self, tension_arm: &TensionArm) {
        let angle = tension_arm.angle();
        let value = self.tension_calculator.calc_filament_tension(angle);
        self.filament_tension.set(value);
    }
}

pub struct SpeedAlgorithmInput {
    pub dt: Duration,
    pub enabled: bool,
    pub speed_min: AngularVelocity,
    pub speed_max: AngularVelocity,
    pub puller_speed: Velocity,
    pub filament_tension: Option<f64>,
}
