use super::prepare_control::{PrepareConfig, PreparePhase};
use qitech_lib::units::{
    angular_velocity::revolution_per_minute, f64::*, velocity::meter_per_minute,
};
use std::f64::consts::PI;
use std::time::{Duration, Instant};

const MILLIMETERS_PER_METER: f64 = 1000.0;
const MAX_CONTROL_DT_S: f64 = 0.2;
const RATIO_LEARNING_DEADBAND_MULTIPLIER: f64 = 3.0;
const RATIO_LEARNING_MAX_COMMAND_FRACTION: f64 = 0.95;
const RATIO_LEARNING_MAX_FILTER_ERROR_DEG: f64 = 6.0;
const RATIO_LEARNING_MAX_STABLE_RATE_DEG_PER_S: f64 = 4.0;
const SOURCE_LOW_RATIO_LEARNING_GAIN: f64 = 0.012;
const SOURCE_HIGH_RATIO_LEARNING_GAIN: f64 = 0.0025;
const TAKEUP_RATIO_LEARNING_GAIN: f64 = 0.003;
const PULLER_RATIO_LEARNING_MAX_ACCEL_M_PER_MIN_S: f64 = 0.2;
const MOVING_LINE_SPEED_THRESHOLD_M_PER_MIN: f64 = 0.05;
const MIN_VALID_ARM_ANGLE_DEG: f64 = -90.0;
const MAX_VALID_ARM_ANGLE_DEG: f64 = 180.0;
const SOURCE_WARNING_LOW_MOVING_FLOOR_FRACTION: f64 = 0.05;
const TAKEUP_LOW_MOVING_FLOOR_FRACTION: f64 = 0.45;
const TAKEUP_WARNING_HIGH_MOVING_FLOOR_FRACTION: f64 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmZone {
    HardLow,
    EmergencyLow,
    WarningLow,
    Comfort,
    WarningHigh,
    EmergencyHigh,
    HardHigh,
}

impl ArmZone {
    pub fn is_fault(self) -> bool {
        matches!(self, Self::HardLow | Self::HardHigh)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ArmConfig {
    pub hard_min_deg: f64,
    pub hard_max_deg: f64,
    pub start_min_deg: f64,
    pub start_max_deg: f64,
    pub warning_min_deg: f64,
    pub warning_max_deg: f64,
    pub emergency_min_deg: f64,
    pub emergency_max_deg: f64,
    pub target_deg: f64,
    pub filter_time_constant_s: f64,
}

impl ArmConfig {
    pub const SOURCE: Self = Self {
        hard_min_deg: 15.0,
        hard_max_deg: 85.0,
        start_min_deg: 35.0,
        start_max_deg: 65.0,
        warning_min_deg: 25.0,
        warning_max_deg: 62.0,
        emergency_min_deg: 22.0,
        emergency_max_deg: 78.0,
        target_deg: 50.0,
        filter_time_constant_s: 0.6,
    };

    pub const TAKEUP: Self = Self {
        hard_min_deg: 20.0,
        hard_max_deg: 90.0,
        start_min_deg: 35.0,
        start_max_deg: 70.0,
        warning_min_deg: 32.0,
        warning_max_deg: 78.0,
        emergency_min_deg: 24.0,
        emergency_max_deg: 86.0,
        target_deg: 55.0,
        filter_time_constant_s: 0.35,
    };

    pub fn classify(self, angle_deg: f64) -> ArmZone {
        if angle_deg < self.hard_min_deg {
            ArmZone::HardLow
        } else if angle_deg > self.hard_max_deg {
            ArmZone::HardHigh
        } else if angle_deg < self.emergency_min_deg {
            ArmZone::EmergencyLow
        } else if angle_deg > self.emergency_max_deg {
            ArmZone::EmergencyHigh
        } else if angle_deg < self.warning_min_deg {
            ArmZone::WarningLow
        } else if angle_deg > self.warning_max_deg {
            ArmZone::WarningHigh
        } else {
            ArmZone::Comfort
        }
    }

    pub fn in_start_range(self, angle_deg: f64) -> bool {
        (self.start_min_deg..=self.start_max_deg).contains(&angle_deg)
    }

    pub fn in_hard_range(self, angle_deg: f64) -> bool {
        (self.hard_min_deg..=self.hard_max_deg).contains(&angle_deg)
    }

    pub fn with_hard_range(self, min_deg: f64, max_deg: f64) -> Option<Self> {
        let config = Self {
            hard_min_deg: min_deg,
            hard_max_deg: max_deg,
            ..self
        };
        config.is_valid().then_some(config)
    }

    pub fn with_start_range(self, min_deg: f64, max_deg: f64) -> Option<Self> {
        let config = Self {
            start_min_deg: min_deg,
            start_max_deg: max_deg,
            ..self
        };
        config.is_valid().then_some(config)
    }

    pub fn with_target(self, target_deg: f64) -> Option<Self> {
        let config = Self { target_deg, ..self };
        config.is_valid().then_some(config)
    }

    fn is_valid(self) -> bool {
        self.hard_min_deg < self.hard_max_deg
            && self.hard_min_deg >= MIN_VALID_ARM_ANGLE_DEG
            && self.hard_max_deg <= MAX_VALID_ARM_ANGLE_DEG
            && self.start_min_deg >= self.hard_min_deg
            && self.start_max_deg <= self.hard_max_deg
            && self.start_min_deg < self.start_max_deg
            && (self.start_min_deg..=self.start_max_deg).contains(&self.target_deg)
            && self.filter_time_constant_s > 0.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ArmState {
    pub raw_deg: f64,
    pub filtered_deg: f64,
    pub rate_deg_per_s: f64,
    pub zone: ArmZone,
    initialized: bool,
}

impl Default for ArmState {
    fn default() -> Self {
        Self {
            raw_deg: 0.0,
            filtered_deg: 0.0,
            rate_deg_per_s: 0.0,
            zone: ArmZone::HardLow,
            initialized: false,
        }
    }
}

impl ArmState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn update(&mut self, config: ArmConfig, raw_deg: f64, dt_s: f64) {
        let previous = self.filtered_deg;
        if !self.initialized || dt_s <= 0.0 {
            self.filtered_deg = raw_deg;
            self.rate_deg_per_s = 0.0;
            self.initialized = true;
        } else {
            let alpha = dt_s / (config.filter_time_constant_s + dt_s);
            self.filtered_deg += (raw_deg - self.filtered_deg) * alpha;
            self.rate_deg_per_s = (self.filtered_deg - previous) / dt_s;
        }
        self.raw_deg = raw_deg;
        self.zone = config.classify(raw_deg);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FollowerConfig {
    pub initial_ratio_rpm_per_m_per_min: f64,
    pub min_ratio_rpm_per_m_per_min: f64,
    pub max_ratio_rpm_per_m_per_min: f64,
    pub kp_rpm_per_deg: f64,
    pub kd_rpm_per_deg_per_s: f64,
    pub deadband_deg: f64,
    pub max_trim_rpm: f64,
    pub max_trim_feed_forward_fraction: f64,
    pub trim_filter_time_constant_s: f64,
    pub min_feed_forward_fraction: f64,
    pub max_rpm: f64,
    pub max_rpm_change_per_s: f64,
    pub emergency_rpm_change_per_s: f64,
    pub learning_min_line_speed_m_per_min: f64,
}

#[derive(Debug, Clone, Copy)]
struct SourceShapeConfig {
    recovery_exit_deg: f64,
    recovery_full_deg: f64,
    recovery_max_multiplier: f64,
    recovery_reduction_multiplier: f64,
    low_max_multiplier: f64,
    low_reduction_multiplier: f64,
    high_min_multiplier: f64,
    high_boost_multiplier: f64,
}

impl SourceShapeConfig {
    const DEFAULT: Self = Self {
        recovery_exit_deg: 48.0,
        recovery_full_deg: 25.0,
        recovery_max_multiplier: 0.70,
        recovery_reduction_multiplier: 0.55,
        low_max_multiplier: 0.25,
        low_reduction_multiplier: 0.20,
        high_min_multiplier: 1.10,
        high_boost_multiplier: 0.20,
    };
}

#[derive(Debug, Clone, Copy)]
struct TakeupShapeConfig {
    low_min_multiplier: f64,
    low_boost_multiplier: f64,
    high_max_multiplier: f64,
    high_reduction_multiplier: f64,
}

impl TakeupShapeConfig {
    const DEFAULT: Self = Self {
        low_min_multiplier: 1.12,
        low_boost_multiplier: 0.38,
        high_max_multiplier: 0.55,
        high_reduction_multiplier: 0.45,
    };
}

impl FollowerConfig {
    pub const SOURCE: Self = Self {
        initial_ratio_rpm_per_m_per_min: 2.2,
        min_ratio_rpm_per_m_per_min: 0.8,
        max_ratio_rpm_per_m_per_min: 6.0,
        kp_rpm_per_deg: 0.70,
        kd_rpm_per_deg_per_s: 0.12,
        deadband_deg: 2.0,
        max_trim_rpm: 34.0,
        max_trim_feed_forward_fraction: 0.55,
        trim_filter_time_constant_s: 0.7,
        min_feed_forward_fraction: 0.25,
        max_rpm: 220.0,
        max_rpm_change_per_s: 32.0,
        emergency_rpm_change_per_s: 55.0,
        learning_min_line_speed_m_per_min: 5.0,
    };

    pub const TAKEUP: Self = Self {
        initial_ratio_rpm_per_m_per_min: 1.75,
        min_ratio_rpm_per_m_per_min: 0.65,
        max_ratio_rpm_per_m_per_min: 5.0,
        kp_rpm_per_deg: 0.48,
        kd_rpm_per_deg_per_s: 0.24,
        deadband_deg: 3.0,
        max_trim_rpm: 28.0,
        max_trim_feed_forward_fraction: 0.45,
        trim_filter_time_constant_s: 0.65,
        min_feed_forward_fraction: 0.15,
        max_rpm: 220.0,
        max_rpm_change_per_s: 38.0,
        emergency_rpm_change_per_s: 80.0,
        learning_min_line_speed_m_per_min: 6.0,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct FollowerState {
    pub ratio_rpm_per_m_per_min: f64,
    pub feed_forward_rpm: f64,
    pub trim_rpm: f64,
    pub target_rpm: f64,
    pub command_rpm: f64,
    pub learning_active: bool,
    ratio_min_rpm_per_m_per_min: f64,
    ratio_max_rpm_per_m_per_min: f64,
    last_diameter_mm: Option<f64>,
}

impl FollowerState {
    pub fn new(config: FollowerConfig) -> Self {
        Self {
            ratio_rpm_per_m_per_min: config.initial_ratio_rpm_per_m_per_min,
            feed_forward_rpm: 0.0,
            trim_rpm: 0.0,
            target_rpm: 0.0,
            command_rpm: 0.0,
            learning_active: false,
            ratio_min_rpm_per_m_per_min: config.min_ratio_rpm_per_m_per_min,
            ratio_max_rpm_per_m_per_min: config.max_ratio_rpm_per_m_per_min,
            last_diameter_mm: None,
        }
    }

    pub fn reset(&mut self, config: FollowerConfig) {
        *self = Self::new(config);
    }

    pub fn force_zero(&mut self) {
        self.feed_forward_rpm = 0.0;
        self.trim_rpm = 0.0;
        self.target_rpm = 0.0;
        self.command_rpm = 0.0;
        self.learning_active = false;
    }

    fn sync_ratio_baseline(&mut self, config: FollowerConfig, diameter_mm: Option<f64>) {
        if self.last_diameter_mm == diameter_mm {
            return;
        }

        let baseline_ratio =
            feed_forward_ratio(diameter_mm, config.initial_ratio_rpm_per_m_per_min);
        self.ratio_rpm_per_m_per_min = baseline_ratio;
        self.ratio_min_rpm_per_m_per_min =
            config.min_ratio_rpm_per_m_per_min.min(baseline_ratio * 0.6);
        self.ratio_max_rpm_per_m_per_min =
            config.max_ratio_rpm_per_m_per_min.max(baseline_ratio * 1.6);
        self.last_diameter_mm = diameter_mm;
    }

    pub fn update_source(
        &mut self,
        config: FollowerConfig,
        arm_config: ArmConfig,
        arm_state: ArmState,
        line_speed_m_per_min: f64,
        diameter_mm: Option<f64>,
        dt_s: f64,
        learning_allowed: bool,
    ) {
        self.sync_ratio_baseline(config, diameter_mm);
        self.feed_forward_rpm =
            feed_forward_rpm(line_speed_m_per_min, self.ratio_rpm_per_m_per_min);

        let signed_error = arm_state.filtered_deg - arm_config.target_deg;
        let derivative_trim = arm_state.rate_deg_per_s * config.kd_rpm_per_deg_per_s;
        self.trim_rpm = filtered_trim_rpm(
            self.trim_rpm,
            signed_error,
            derivative_trim,
            self.feed_forward_rpm,
            config,
            dt_s,
        );
        let moving_floor_fraction = match arm_state.zone {
            ArmZone::EmergencyLow => 0.0,
            ArmZone::WarningLow => SOURCE_WARNING_LOW_MOVING_FLOOR_FRACTION,
            _ => config.min_feed_forward_fraction,
        };
        let moving_floor_rpm = if line_speed_m_per_min > MOVING_LINE_SPEED_THRESHOLD_M_PER_MIN {
            self.feed_forward_rpm * moving_floor_fraction
        } else {
            0.0
        };
        let target_rpm = self.feed_forward_rpm + self.trim_rpm;
        self.target_rpm = source_target_rpm(
            target_rpm,
            self.feed_forward_rpm,
            moving_floor_rpm,
            arm_state,
            arm_config,
            config,
        );
        let max_change = if self.target_rpm < self.command_rpm
            && matches!(arm_state.zone, ArmZone::WarningLow | ArmZone::EmergencyLow)
        {
            config.emergency_rpm_change_per_s
        } else {
            config.max_rpm_change_per_s
        };
        self.command_rpm = move_toward(self.command_rpm, self.target_rpm, max_change, dt_s);
        self.command_rpm = self.command_rpm.clamp(0.0, config.max_rpm);

        self.learning_active = can_learn_ratio(
            line_speed_m_per_min,
            arm_state,
            self.command_rpm,
            config,
            dt_s,
        );

        if self.learning_active {
            let persistent_error = deadband(
                arm_state.filtered_deg - arm_config.target_deg,
                config.deadband_deg * RATIO_LEARNING_DEADBAND_MULTIPLIER,
            );
            let learning_gain = if persistent_error < 0.0 {
                SOURCE_LOW_RATIO_LEARNING_GAIN
            } else if learning_allowed
                && arm_state.rate_deg_per_s.abs() <= RATIO_LEARNING_MAX_STABLE_RATE_DEG_PER_S
            {
                SOURCE_HIGH_RATIO_LEARNING_GAIN
            } else {
                0.0
            };
            self.update_ratio(persistent_error, learning_gain, dt_s);
        }
    }

    pub fn update_takeup(
        &mut self,
        config: FollowerConfig,
        arm_config: ArmConfig,
        arm_state: ArmState,
        line_speed_m_per_min: f64,
        diameter_mm: Option<f64>,
        dt_s: f64,
        learning_allowed: bool,
    ) {
        self.sync_ratio_baseline(config, diameter_mm);
        self.feed_forward_rpm =
            feed_forward_rpm(line_speed_m_per_min, self.ratio_rpm_per_m_per_min);

        let signed_error = arm_config.target_deg - arm_state.filtered_deg;
        let derivative_trim = -arm_state.rate_deg_per_s * config.kd_rpm_per_deg_per_s;
        self.trim_rpm = filtered_trim_rpm(
            self.trim_rpm,
            signed_error,
            derivative_trim,
            self.feed_forward_rpm,
            config,
            dt_s,
        );

        let moving_floor_fraction = match arm_state.zone {
            ArmZone::EmergencyLow | ArmZone::WarningLow => TAKEUP_LOW_MOVING_FLOOR_FRACTION,
            ArmZone::EmergencyHigh => 0.0,
            ArmZone::WarningHigh => TAKEUP_WARNING_HIGH_MOVING_FLOOR_FRACTION,
            _ => config.min_feed_forward_fraction,
        };
        let moving_floor_rpm = if line_speed_m_per_min > MOVING_LINE_SPEED_THRESHOLD_M_PER_MIN {
            self.feed_forward_rpm * moving_floor_fraction
        } else {
            0.0
        };
        let target_rpm = self.feed_forward_rpm + self.trim_rpm;
        self.target_rpm = takeup_target_rpm(
            target_rpm,
            self.feed_forward_rpm,
            moving_floor_rpm,
            arm_state,
            arm_config,
            config,
        );

        self.command_rpm = move_toward(
            self.command_rpm,
            self.target_rpm,
            config.max_rpm_change_per_s,
            dt_s,
        );
        self.command_rpm = self.command_rpm.clamp(0.0, config.max_rpm);

        self.learning_active = can_learn_ratio(
            line_speed_m_per_min,
            arm_state,
            self.command_rpm,
            config,
            dt_s,
        );

        if self.learning_active
            && learning_allowed
            && arm_state.rate_deg_per_s.abs() <= RATIO_LEARNING_MAX_STABLE_RATE_DEG_PER_S
        {
            let persistent_error = deadband(
                arm_config.target_deg - arm_state.filtered_deg,
                config.deadband_deg * RATIO_LEARNING_DEADBAND_MULTIPLIER,
            );
            self.update_ratio(persistent_error, TAKEUP_RATIO_LEARNING_GAIN, dt_s);
        }
    }

    fn update_ratio(&mut self, persistent_error_deg: f64, learning_gain: f64, dt_s: f64) {
        let ratio_step = persistent_error_deg * learning_gain * dt_s;
        self.ratio_rpm_per_m_per_min = (self.ratio_rpm_per_m_per_min + ratio_step).clamp(
            self.ratio_min_rpm_per_m_per_min,
            self.ratio_max_rpm_per_m_per_min,
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PullerRampConfig {
    pub crawl_speed_m_per_min: f64,
    pub normal_accel_m_per_min_s: f64,
    pub warning_accel_m_per_min_s: f64,
    pub normal_decel_m_per_min_s: f64,
}

impl Default for PullerRampConfig {
    fn default() -> Self {
        Self {
            crawl_speed_m_per_min: 1.0,
            normal_accel_m_per_min_s: 5.0,
            warning_accel_m_per_min_s: 0.75,
            normal_decel_m_per_min_s: 5.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RewindControlConfig {
    pub source_arm: ArmConfig,
    pub takeup_arm: ArmConfig,
    pub source_follower: FollowerConfig,
    pub takeup_follower: FollowerConfig,
    pub puller_ramp: PullerRampConfig,
    pub prepare: PrepareConfig,
    pub precharge_duration: Duration,
    pub crawl_duration: Duration,
}

impl Default for RewindControlConfig {
    fn default() -> Self {
        Self {
            source_arm: ArmConfig::SOURCE,
            takeup_arm: ArmConfig::TAKEUP,
            source_follower: FollowerConfig::SOURCE,
            takeup_follower: FollowerConfig::TAKEUP,
            puller_ramp: PullerRampConfig::default(),
            prepare: PrepareConfig::default(),
            precharge_duration: Duration::from_secs(1),
            crawl_duration: Duration::from_secs(3),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RewindControlState {
    pub config: RewindControlConfig,
    pub source_arm: ArmState,
    pub takeup_arm: ArmState,
    pub source_follower: FollowerState,
    pub takeup_follower: FollowerState,
    pub puller_command_m_per_min: f64,
    pub puller_accel_m_per_min_s: f64,
    pub last_dt_s: f64,
    pub last_update: Option<Instant>,
    pub phase_started_at: Option<Instant>,
    pub prepare_settled_since: Option<Instant>,
    pub prepare_phase: PreparePhase,
    pub(super) prepare_phase_started_at: Option<Instant>,
}

impl RewindControlState {
    pub fn new(config: RewindControlConfig) -> Self {
        Self {
            source_arm: ArmState::default(),
            takeup_arm: ArmState::default(),
            source_follower: FollowerState::new(config.source_follower),
            takeup_follower: FollowerState::new(config.takeup_follower),
            config,
            puller_command_m_per_min: 0.0,
            puller_accel_m_per_min_s: 0.0,
            last_dt_s: 0.0,
            last_update: None,
            phase_started_at: None,
            prepare_settled_since: None,
            prepare_phase: PreparePhase::Relax,
            prepare_phase_started_at: None,
        }
    }

    pub fn reset_for_rewind(&mut self, now: Instant) {
        self.source_arm.reset();
        self.takeup_arm.reset();
        self.source_follower.reset(self.config.source_follower);
        self.takeup_follower.reset(self.config.takeup_follower);
        self.puller_command_m_per_min = 0.0;
        self.puller_accel_m_per_min_s = 0.0;
        self.last_dt_s = 0.0;
        self.last_update = Some(now);
        self.phase_started_at = Some(now);
        self.prepare_settled_since = None;
        self.prepare_phase = PreparePhase::Relax;
        self.prepare_phase_started_at = None;
    }

    pub fn reset_motion(&mut self) {
        self.source_follower.force_zero();
        self.takeup_follower.force_zero();
        self.puller_command_m_per_min = 0.0;
        self.puller_accel_m_per_min_s = 0.0;
        self.last_dt_s = 0.0;
        self.last_update = None;
        self.phase_started_at = None;
        self.prepare_settled_since = None;
        self.prepare_phase = PreparePhase::Relax;
        self.prepare_phase_started_at = None;
    }

    pub fn decelerate_motion(&mut self, dt_s: f64) {
        let rpm_rate = self
            .config
            .source_follower
            .emergency_rpm_change_per_s
            .max(self.config.prepare.max_rpm_change_per_s);
        self.source_follower.command_rpm =
            move_toward(self.source_follower.command_rpm, 0.0, rpm_rate, dt_s);
        self.source_follower.target_rpm = 0.0;
        self.source_follower.trim_rpm = 0.0;
        self.source_follower.feed_forward_rpm = 0.0;
        self.source_follower.learning_active = false;

        self.takeup_follower.command_rpm = move_toward(
            self.takeup_follower.command_rpm,
            0.0,
            self.config.prepare.max_rpm_change_per_s,
            dt_s,
        );
        self.takeup_follower.target_rpm = 0.0;
        self.takeup_follower.trim_rpm = 0.0;
        self.takeup_follower.feed_forward_rpm = 0.0;
        self.puller_command_m_per_min = move_toward(
            self.puller_command_m_per_min,
            0.0,
            self.config.puller_ramp.normal_decel_m_per_min_s,
            dt_s,
        );
    }

    pub fn decelerate_motion_at(&mut self, now: Instant) {
        let dt_s = self
            .last_update
            .map(|last| now.duration_since(last).as_secs_f64().min(MAX_CONTROL_DT_S))
            .unwrap_or(0.0);
        self.last_update = Some(now);
        self.last_dt_s = dt_s;
        self.decelerate_motion(dt_s);
    }

    pub fn update_arms(
        &mut self,
        source_angle_deg: f64,
        takeup_angle_deg: f64,
        now: Instant,
    ) -> f64 {
        let dt_s = self
            .last_update
            .map(|last| now.duration_since(last).as_secs_f64().min(MAX_CONTROL_DT_S))
            .unwrap_or(0.0);
        self.source_arm
            .update(self.config.source_arm, source_angle_deg, dt_s);
        self.takeup_arm
            .update(self.config.takeup_arm, takeup_angle_deg, dt_s);
        self.last_update = Some(now);
        self.last_dt_s = dt_s;
        dt_s
    }

    pub fn start_phase(&mut self, now: Instant) {
        self.phase_started_at = Some(now);
    }

    pub fn phase_elapsed(&self, now: Instant) -> Duration {
        self.phase_started_at
            .map(|started| now.duration_since(started))
            .unwrap_or_default()
    }

    pub fn puller_command_speed(&self) -> Velocity {
        Velocity::new::<meter_per_minute>(self.puller_command_m_per_min)
    }

    pub fn source_command_angular_velocity(&self) -> AngularVelocity {
        AngularVelocity::new::<revolution_per_minute>(self.source_follower.command_rpm)
    }

    pub fn takeup_command_angular_velocity(&self) -> AngularVelocity {
        AngularVelocity::new::<revolution_per_minute>(self.takeup_follower.command_rpm)
    }

    pub fn update_puller_command(&mut self, target: Velocity, dt_s: f64) {
        let ramp = self.config.puller_ramp;
        let constrained_target = target.get::<meter_per_minute>();

        let rate = if constrained_target >= self.puller_command_m_per_min {
            let source_scale =
                low_side_accel_scale(self.source_arm.filtered_deg, self.config.source_arm);
            let takeup_scale =
                low_side_accel_scale(self.takeup_arm.filtered_deg, self.config.takeup_arm);
            let source_high_scale =
                high_side_accel_scale(self.source_arm.filtered_deg, self.config.source_arm);
            let takeup_high_scale =
                high_side_accel_scale(self.takeup_arm.filtered_deg, self.config.takeup_arm);
            (ramp.normal_accel_m_per_min_s
                * source_scale
                    .min(takeup_scale)
                    .min(source_high_scale)
                    .min(takeup_high_scale))
            .max(ramp.warning_accel_m_per_min_s)
        } else {
            ramp.normal_decel_m_per_min_s
        };

        let previous = self.puller_command_m_per_min;
        let max_delta = rate * dt_s.max(0.0);
        self.puller_command_m_per_min +=
            (constrained_target - self.puller_command_m_per_min).clamp(-max_delta, max_delta);
        self.puller_command_m_per_min = self.puller_command_m_per_min.max(0.0);
        self.puller_accel_m_per_min_s = if dt_s > 0.0 {
            (self.puller_command_m_per_min - previous) / dt_s
        } else {
            0.0
        };
    }

    pub fn update_followers(
        &mut self,
        line_speed: Velocity,
        takeup_diameter_mm: Option<f64>,
        source_diameter_mm: Option<f64>,
        dt_s: f64,
    ) {
        let line_speed_m_per_min = line_speed.get::<meter_per_minute>();
        let learning_allowed =
            self.puller_accel_m_per_min_s.abs() <= PULLER_RATIO_LEARNING_MAX_ACCEL_M_PER_MIN_S;
        self.source_follower.update_source(
            self.config.source_follower,
            self.config.source_arm,
            self.source_arm,
            line_speed_m_per_min,
            source_diameter_mm,
            dt_s,
            learning_allowed,
        );
        self.takeup_follower.update_takeup(
            self.config.takeup_follower,
            self.config.takeup_arm,
            self.takeup_arm,
            line_speed_m_per_min,
            takeup_diameter_mm,
            dt_s,
            learning_allowed,
        );
    }

    pub fn source_recovery_active(&self) -> bool {
        source_needs_recovery(self.source_arm)
    }
}

pub(crate) fn deadband(value: f64, width: f64) -> f64 {
    if value.abs() <= width {
        0.0
    } else {
        value.signum() * (value.abs() - width)
    }
}

pub(crate) fn move_toward(current: f64, target: f64, rate_per_s: f64, dt_s: f64) -> f64 {
    let max_delta = rate_per_s * dt_s.max(0.0);
    current + (target - current).clamp(-max_delta, max_delta)
}

fn feed_forward_ratio(diameter_mm: Option<f64>, fallback_ratio_rpm_per_m_per_min: f64) -> f64 {
    match diameter_mm {
        Some(diameter_mm) if diameter_mm > 0.0 => MILLIMETERS_PER_METER / (PI * diameter_mm),
        _ => fallback_ratio_rpm_per_m_per_min,
    }
}

fn feed_forward_rpm(line_speed_m_per_min: f64, ratio_rpm_per_m_per_min: f64) -> f64 {
    line_speed_m_per_min.max(0.0) * ratio_rpm_per_m_per_min
}

fn filtered_trim_rpm(
    current_trim_rpm: f64,
    signed_error_deg: f64,
    derivative_trim_rpm: f64,
    feed_forward_rpm: f64,
    config: FollowerConfig,
    dt_s: f64,
) -> f64 {
    let effective_error_deg = deadband(signed_error_deg, config.deadband_deg);
    let trim_limit_rpm = config
        .max_trim_rpm
        .max(feed_forward_rpm * config.max_trim_feed_forward_fraction);
    let raw_trim_rpm = (effective_error_deg * config.kp_rpm_per_deg + derivative_trim_rpm)
        .clamp(-trim_limit_rpm, trim_limit_rpm);

    if config.trim_filter_time_constant_s > 0.0 && dt_s > 0.0 {
        let alpha = dt_s / (config.trim_filter_time_constant_s + dt_s);
        current_trim_rpm + (raw_trim_rpm - current_trim_rpm) * alpha
    } else {
        raw_trim_rpm
    }
}

fn can_learn_ratio(
    line_speed_m_per_min: f64,
    arm_state: ArmState,
    command_rpm: f64,
    config: FollowerConfig,
    dt_s: f64,
) -> bool {
    line_speed_m_per_min >= config.learning_min_line_speed_m_per_min
        && !matches!(arm_state.zone, ArmZone::HardLow | ArmZone::HardHigh)
        && (arm_state.raw_deg - arm_state.filtered_deg).abs() <= RATIO_LEARNING_MAX_FILTER_ERROR_DEG
        && command_rpm < config.max_rpm * RATIO_LEARNING_MAX_COMMAND_FRACTION
        && dt_s > 0.0
}

fn low_side_accel_scale(angle_deg: f64, config: ArmConfig) -> f64 {
    low_accel_scale(angle_deg, config.warning_min_deg, config.emergency_min_deg)
}

fn high_side_accel_scale(angle_deg: f64, config: ArmConfig) -> f64 {
    high_accel_scale(angle_deg, config.warning_max_deg, config.emergency_max_deg)
}

fn low_accel_scale(angle_deg: f64, warning_deg: f64, emergency_deg: f64) -> f64 {
    const MIN_WARNING_ACCEL_SCALE: f64 = 0.15;

    let progress = if angle_deg >= warning_deg {
        0.0
    } else {
        (warning_deg - angle_deg) / (warning_deg - emergency_deg).max(f64::EPSILON)
    };

    1.0 - (1.0 - MIN_WARNING_ACCEL_SCALE) * progress.clamp(0.0, 1.0)
}

fn high_accel_scale(angle_deg: f64, warning_deg: f64, emergency_deg: f64) -> f64 {
    const MIN_WARNING_ACCEL_SCALE: f64 = 0.15;

    let progress = if angle_deg <= warning_deg {
        0.0
    } else {
        (angle_deg - warning_deg) / (emergency_deg - warning_deg).max(f64::EPSILON)
    };

    1.0 - (1.0 - MIN_WARNING_ACCEL_SCALE) * progress.clamp(0.0, 1.0)
}

fn source_low_recovery_angle(source_arm: ArmState) -> f64 {
    source_arm.raw_deg.min(source_arm.filtered_deg)
}

fn source_low_recovery_progress(source_arm: ArmState) -> f64 {
    let config = SourceShapeConfig::DEFAULT;
    let recovery_angle_deg = source_low_recovery_angle(source_arm);
    let recovery_span = (config.recovery_exit_deg - config.recovery_full_deg).max(f64::EPSILON);
    ((config.recovery_exit_deg - recovery_angle_deg) / recovery_span).clamp(0.0, 1.0)
}

fn source_needs_recovery(source_arm: ArmState) -> bool {
    source_low_recovery_angle(source_arm) < SourceShapeConfig::DEFAULT.recovery_exit_deg
}

fn source_target_rpm(
    target_rpm: f64,
    feed_forward_rpm: f64,
    moving_floor_rpm: f64,
    arm_state: ArmState,
    arm_config: ArmConfig,
    follower_config: FollowerConfig,
) -> f64 {
    if feed_forward_rpm <= 0.0 {
        return 0.0;
    }

    let shape = SourceShapeConfig::DEFAULT;
    let mut multiplier = target_rpm / feed_forward_rpm;

    if source_needs_recovery(arm_state) {
        let recovery_progress = source_low_recovery_progress(arm_state);
        let recovery_max_multiplier =
            shape.recovery_max_multiplier - shape.recovery_reduction_multiplier * recovery_progress;
        multiplier = multiplier.min(recovery_max_multiplier);
    }

    if arm_state.raw_deg < arm_config.warning_min_deg {
        let low_span = (arm_config.warning_min_deg - arm_config.hard_min_deg).max(f64::EPSILON);
        let low_progress =
            ((arm_config.warning_min_deg - arm_state.raw_deg) / low_span).clamp(0.0, 1.0);
        multiplier = multiplier
            .min(shape.low_max_multiplier - shape.low_reduction_multiplier * low_progress);
    } else if arm_state.filtered_deg > arm_config.warning_max_deg {
        let high_span = (arm_config.hard_max_deg - arm_config.warning_max_deg).max(f64::EPSILON);
        let high_progress =
            ((arm_state.filtered_deg - arm_config.warning_max_deg) / high_span).clamp(0.0, 1.0);
        multiplier =
            multiplier.max(shape.high_min_multiplier + shape.high_boost_multiplier * high_progress);
    }

    (feed_forward_rpm * multiplier).clamp(moving_floor_rpm, follower_config.max_rpm)
}

fn takeup_target_rpm(
    target_rpm: f64,
    feed_forward_rpm: f64,
    moving_floor_rpm: f64,
    arm_state: ArmState,
    arm_config: ArmConfig,
    follower_config: FollowerConfig,
) -> f64 {
    if feed_forward_rpm <= 0.0 {
        return 0.0;
    }

    let shape = TakeupShapeConfig::DEFAULT;
    let mut multiplier = target_rpm / feed_forward_rpm;

    if arm_state.raw_deg < arm_config.warning_min_deg {
        let low_span = (arm_config.warning_min_deg - arm_config.hard_min_deg).max(f64::EPSILON);
        let low_progress =
            ((arm_config.warning_min_deg - arm_state.raw_deg) / low_span).clamp(0.0, 1.0);
        multiplier =
            multiplier.max(shape.low_min_multiplier + shape.low_boost_multiplier * low_progress);
    } else if arm_state.raw_deg > arm_config.warning_max_deg {
        let high_span = (arm_config.hard_max_deg - arm_config.warning_max_deg).max(f64::EPSILON);
        let high_progress =
            ((arm_state.raw_deg - arm_config.warning_max_deg) / high_span).clamp(0.0, 1.0);
        multiplier = multiplier
            .min(shape.high_max_multiplier - shape.high_reduction_multiplier * high_progress);
    }

    (feed_forward_rpm * multiplier).clamp(moving_floor_rpm, follower_config.max_rpm)
}
