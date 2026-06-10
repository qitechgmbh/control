use std::time::{Duration, Instant};

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

    pub fn is_emergency(self) -> bool {
        matches!(self, Self::EmergencyLow | Self::EmergencyHigh)
    }

    pub fn is_warning_or_worse(self) -> bool {
        !matches!(self, Self::Comfort)
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
pub enum FollowerResponse {
    Source,
    Takeup,
}

#[derive(Debug, Clone, Copy)]
pub struct FollowerConfig {
    pub response: FollowerResponse,
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
    pub slew_rpm_per_s: f64,
    pub emergency_slew_rpm_per_s: f64,
    pub learning_min_line_speed_m_per_min: f64,
    pub learning_tau_s: f64,
}

impl FollowerConfig {
    pub const SOURCE: Self = Self {
        response: FollowerResponse::Source,
        initial_ratio_rpm_per_m_per_min: 2.75,
        min_ratio_rpm_per_m_per_min: 1.2,
        max_ratio_rpm_per_m_per_min: 6.0,
        kp_rpm_per_deg: 0.45,
        kd_rpm_per_deg_per_s: 0.12,
        deadband_deg: 2.0,
        max_trim_rpm: 32.0,
        max_trim_feed_forward_fraction: 0.35,
        trim_filter_time_constant_s: 0.8,
        min_feed_forward_fraction: 0.45,
        max_rpm: 220.0,
        slew_rpm_per_s: 35.0,
        emergency_slew_rpm_per_s: 55.0,
        learning_min_line_speed_m_per_min: 5.0,
        learning_tau_s: 25.0,
    };

    pub const TAKEUP: Self = Self {
        response: FollowerResponse::Takeup,
        initial_ratio_rpm_per_m_per_min: 3.0,
        min_ratio_rpm_per_m_per_min: 0.5,
        max_ratio_rpm_per_m_per_min: 8.0,
        kp_rpm_per_deg: 0.45,
        kd_rpm_per_deg_per_s: 0.0,
        deadband_deg: 2.0,
        max_trim_rpm: 10.0,
        max_trim_feed_forward_fraction: 0.20,
        trim_filter_time_constant_s: 0.0,
        min_feed_forward_fraction: 0.35,
        max_rpm: 110.0,
        slew_rpm_per_s: 22.0,
        emergency_slew_rpm_per_s: 22.0,
        learning_min_line_speed_m_per_min: 5.0,
        learning_tau_s: 60.0,
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

    pub fn update(
        &mut self,
        config: FollowerConfig,
        arm_config: ArmConfig,
        arm_state: ArmState,
        line_speed_m_per_min: f64,
        dt_s: f64,
        learning_allowed: bool,
    ) {
        self.feed_forward_rpm = line_speed_m_per_min.max(0.0) * self.ratio_rpm_per_m_per_min;

        let signed_error = match config.response {
            FollowerResponse::Source => arm_state.filtered_deg - arm_config.target_deg,
            FollowerResponse::Takeup => arm_config.target_deg - arm_state.filtered_deg,
        };
        let effective_error = deadband(signed_error, config.deadband_deg);
        let derivative_trim = match config.response {
            FollowerResponse::Source => arm_state.rate_deg_per_s * config.kd_rpm_per_deg_per_s,
            FollowerResponse::Takeup => -arm_state.rate_deg_per_s * config.kd_rpm_per_deg_per_s,
        };
        let trim_limit_rpm = config
            .max_trim_rpm
            .max(self.feed_forward_rpm * config.max_trim_feed_forward_fraction);
        let raw_trim_rpm = (effective_error * config.kp_rpm_per_deg + derivative_trim)
            .clamp(-trim_limit_rpm, trim_limit_rpm);
        if config.trim_filter_time_constant_s > 0.0 && dt_s > 0.0 {
            let alpha = dt_s / (config.trim_filter_time_constant_s + dt_s);
            self.trim_rpm += (raw_trim_rpm - self.trim_rpm) * alpha;
        } else {
            self.trim_rpm = raw_trim_rpm;
        }
        let moving_floor_rpm = if line_speed_m_per_min > 0.05 {
            self.feed_forward_rpm * config.min_feed_forward_fraction
        } else {
            0.0
        };
        let target_rpm = self.feed_forward_rpm + self.trim_rpm;
        self.target_rpm = match config.response {
            FollowerResponse::Source => source_target_rpm(
                target_rpm,
                self.feed_forward_rpm,
                arm_state,
                arm_config,
                config,
            ),
            FollowerResponse::Takeup => target_rpm.clamp(moving_floor_rpm, config.max_rpm),
        };
        let slew_rpm_per_s = if matches!(config.response, FollowerResponse::Source)
            && self.target_rpm < self.command_rpm
            && matches!(arm_state.zone, ArmZone::WarningLow | ArmZone::EmergencyLow)
        {
            config.emergency_slew_rpm_per_s
        } else {
            config.slew_rpm_per_s
        };
        let max_delta = slew_rpm_per_s * dt_s.max(0.0);
        self.command_rpm += (self.target_rpm - self.command_rpm).clamp(-max_delta, max_delta);
        self.command_rpm = self.command_rpm.clamp(0.0, config.max_rpm);

        self.learning_active = !matches!(config.response, FollowerResponse::Source)
            && learning_allowed
            && line_speed_m_per_min >= config.learning_min_line_speed_m_per_min
            && arm_state.zone == ArmZone::Comfort
            && arm_state.rate_deg_per_s.abs() <= 2.0
            && self.trim_rpm.abs() <= trim_limit_rpm * 0.35;

        if self.learning_active && line_speed_m_per_min > 0.0 && dt_s > 0.0 {
            let observed_ratio = (self.command_rpm / line_speed_m_per_min).clamp(
                config.min_ratio_rpm_per_m_per_min,
                config.max_ratio_rpm_per_m_per_min,
            );
            let alpha = (dt_s / config.learning_tau_s).clamp(0.0, 1.0);
            self.ratio_rpm_per_m_per_min += (observed_ratio - self.ratio_rpm_per_m_per_min) * alpha;
        }

        if matches!(config.response, FollowerResponse::Source)
            && learning_allowed
            && line_speed_m_per_min >= config.learning_min_line_speed_m_per_min
            && !source_needs_recovery(arm_state)
            && !matches!(arm_state.zone, ArmZone::HardLow | ArmZone::HardHigh)
            && arm_state.rate_deg_per_s.abs() <= 4.0
            && (arm_state.raw_deg - arm_state.filtered_deg).abs() <= 6.0
            && self.command_rpm < config.max_rpm * 0.95
            && dt_s > 0.0
        {
            let persistent_error = deadband(
                arm_state.filtered_deg - arm_config.target_deg,
                config.deadband_deg * 3.0,
            );
            let ratio_step = persistent_error * 0.0025 * dt_s;
            self.ratio_rpm_per_m_per_min = (self.ratio_rpm_per_m_per_min + ratio_step).clamp(
                config.min_ratio_rpm_per_m_per_min,
                config.max_ratio_rpm_per_m_per_min,
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PullerRampConfig {
    pub crawl_speed_m_per_min: f64,
    pub normal_accel_m_per_min_s: f64,
    pub source_recovery_accel_min_m_per_min_s: f64,
    pub source_recovery_accel_max_m_per_min_s: f64,
    pub warning_accel_m_per_min_s: f64,
    pub emergency_decel_m_per_min_s: f64,
    pub normal_decel_m_per_min_s: f64,
}

impl Default for PullerRampConfig {
    fn default() -> Self {
        Self {
            crawl_speed_m_per_min: 1.0,
            normal_accel_m_per_min_s: 5.0,
            source_recovery_accel_min_m_per_min_s: 0.35,
            source_recovery_accel_max_m_per_min_s: 1.25,
            warning_accel_m_per_min_s: 0.75,
            emergency_decel_m_per_min_s: 5.0,
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
    }

    pub fn reset_motion(&mut self) {
        self.source_follower.force_zero();
        self.takeup_follower.force_zero();
        self.puller_command_m_per_min = 0.0;
        self.puller_accel_m_per_min_s = 0.0;
        self.last_dt_s = 0.0;
        self.last_update = None;
        self.phase_started_at = None;
    }

    pub fn update_arms(
        &mut self,
        source_angle_deg: f64,
        takeup_angle_deg: f64,
        now: Instant,
    ) -> f64 {
        let dt_s = self
            .last_update
            .map(|last| now.duration_since(last).as_secs_f64().min(0.2))
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

    pub fn update_puller_command(&mut self, target_m_per_min: f64, dt_s: f64) {
        let ramp = self.config.puller_ramp;
        let source_recovery = source_needs_recovery(self.source_arm);
        let constrained_target = target_m_per_min;

        let rate = if constrained_target >= self.puller_command_m_per_min {
            if source_recovery {
                let recovery_progress = source_low_recovery_progress(self.source_arm);
                ramp.source_recovery_accel_min_m_per_min_s
                    + (ramp.source_recovery_accel_max_m_per_min_s
                        - ramp.source_recovery_accel_min_m_per_min_s)
                        * (1.0 - recovery_progress)
            } else {
                let source_scale =
                    low_side_accel_scale(self.source_arm.filtered_deg, self.config.source_arm);
                let takeup_scale =
                    low_side_accel_scale(self.takeup_arm.filtered_deg, self.config.takeup_arm);
                let source_tracking_scale = source_tracking_accel_scale(
                    self.source_arm,
                    self.source_follower.target_rpm,
                    self.source_follower.command_rpm,
                );
                (ramp.normal_accel_m_per_min_s
                    * source_scale.min(takeup_scale).min(source_tracking_scale))
                .max(ramp.warning_accel_m_per_min_s)
            }
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

    pub fn update_followers(&mut self, line_speed_m_per_min: f64, dt_s: f64) {
        let learning_allowed = self.puller_accel_m_per_min_s.abs() <= 0.2;
        self.source_follower.update(
            self.config.source_follower,
            self.config.source_arm,
            self.source_arm,
            line_speed_m_per_min,
            dt_s,
            learning_allowed,
        );
        self.takeup_follower.force_zero();
    }

    pub fn source_recovery_active(&self) -> bool {
        source_needs_recovery(self.source_arm)
    }
}

fn deadband(value: f64, width: f64) -> f64 {
    if value.abs() <= width {
        0.0
    } else {
        value.signum() * (value.abs() - width)
    }
}

fn low_side_accel_scale(angle_deg: f64, config: ArmConfig) -> f64 {
    low_accel_scale(angle_deg, config.warning_min_deg, config.emergency_min_deg)
}

fn low_accel_scale(angle_deg: f64, warning_deg: f64, emergency_deg: f64) -> f64 {
    const MIN_SCALE: f64 = 0.15;

    let progress = if angle_deg >= warning_deg {
        0.0
    } else {
        (warning_deg - angle_deg) / (warning_deg - emergency_deg).max(f64::EPSILON)
    };

    1.0 - (1.0 - MIN_SCALE) * progress.clamp(0.0, 1.0)
}

fn source_tracking_accel_scale(
    source_arm: ArmState,
    source_target_rpm: f64,
    source_command_rpm: f64,
) -> f64 {
    const MIN_SCALE: f64 = 0.18;

    let target_gap_rpm = (source_target_rpm - source_command_rpm).abs();
    let target_gap_scale = 1.0 - (target_gap_rpm / 35.0).clamp(0.0, 1.0) * (1.0 - MIN_SCALE);

    let angle_disagreement_deg = (source_arm.raw_deg - source_arm.filtered_deg).abs();
    let disagreement_scale =
        1.0 - ((angle_disagreement_deg - 8.0) / 18.0).clamp(0.0, 1.0) * (1.0 - MIN_SCALE);

    let rate_scale =
        1.0 - ((source_arm.rate_deg_per_s.abs() - 12.0) / 28.0).clamp(0.0, 1.0) * (1.0 - MIN_SCALE);

    target_gap_scale.min(disagreement_scale).min(rate_scale)
}

const SOURCE_LOW_RECOVERY_EXIT_DEG: f64 = 48.0;
const SOURCE_LOW_FULL_RECOVERY_DEG: f64 = 25.0;

fn source_low_recovery_angle(source_arm: ArmState) -> f64 {
    source_arm.raw_deg.min(source_arm.filtered_deg)
}

fn source_low_recovery_progress(source_arm: ArmState) -> f64 {
    let recovery_angle_deg = source_low_recovery_angle(source_arm);
    let recovery_span =
        (SOURCE_LOW_RECOVERY_EXIT_DEG - SOURCE_LOW_FULL_RECOVERY_DEG).max(f64::EPSILON);
    ((SOURCE_LOW_RECOVERY_EXIT_DEG - recovery_angle_deg) / recovery_span).clamp(0.0, 1.0)
}

fn source_needs_recovery(source_arm: ArmState) -> bool {
    source_low_recovery_angle(source_arm) < SOURCE_LOW_RECOVERY_EXIT_DEG
}

fn source_target_rpm(
    _target_rpm: f64,
    feed_forward_rpm: f64,
    arm_state: ArmState,
    arm_config: ArmConfig,
    follower_config: FollowerConfig,
) -> f64 {
    if feed_forward_rpm <= 0.0 {
        return 0.0;
    }

    let angle_error_deg = deadband(
        arm_state.filtered_deg - arm_config.target_deg,
        follower_config.deadband_deg,
    );
    let angle_fraction = angle_error_deg * 0.025;
    let mut multiplier = (1.0 + angle_fraction).clamp(0.45, 1.30);

    if source_needs_recovery(arm_state) {
        let recovery_progress = source_low_recovery_progress(arm_state);
        let recovery_max_multiplier = 0.80 - 0.45 * recovery_progress;
        multiplier = multiplier.min(recovery_max_multiplier);
    }

    if arm_state.raw_deg < arm_config.warning_min_deg {
        let low_span = (arm_config.warning_min_deg - arm_config.hard_min_deg).max(f64::EPSILON);
        let low_progress =
            ((arm_config.warning_min_deg - arm_state.raw_deg) / low_span).clamp(0.0, 1.0);
        multiplier = multiplier.min(0.35 - 0.30 * low_progress);
    } else if arm_state.filtered_deg > arm_config.warning_max_deg {
        let high_span = (arm_config.hard_max_deg - arm_config.warning_max_deg).max(f64::EPSILON);
        let high_progress =
            ((arm_state.filtered_deg - arm_config.warning_max_deg) / high_span).clamp(0.0, 1.0);
        multiplier = multiplier.max(1.10 + 0.20 * high_progress);
    }

    (feed_forward_rpm * multiplier).clamp(0.0, follower_config.max_rpm)
}
