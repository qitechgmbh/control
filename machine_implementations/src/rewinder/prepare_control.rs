use super::{
    Rewinder, RewinderMode,
    rewind_control::{RewindControlState, deadband, move_toward},
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct PrepareConfig {
    pub settle_tolerance_deg: f64,
    pub settle_rate_deg_per_s: f64,
    pub settle_duration: Duration,
    pub relax_duration: Duration,
    pub puller_kp_m_per_min_per_deg: f64,
    pub puller_max_speed_change_m_per_min_s: f64,
    pub puller_max_m_per_min: f64,
    pub source_kp_rpm_per_deg: f64,
    pub source_kd_rpm_per_deg_per_s: f64,
    pub takeup_kp_rpm_per_deg: f64,
    pub takeup_kd_rpm_per_deg_per_s: f64,
    pub source_max_rpm: f64,
    pub takeup_max_rpm: f64,
    pub max_rpm_change_per_s: f64,
    pub takeup_puller_inhibit_deg: f64,
    pub source_high_puller_inhibit_deg: f64,
}

impl Default for PrepareConfig {
    fn default() -> Self {
        Self {
            settle_tolerance_deg: 6.0,
            settle_rate_deg_per_s: 5.0,
            settle_duration: Duration::from_millis(150),
            relax_duration: Duration::from_millis(150),
            puller_kp_m_per_min_per_deg: 0.018,
            puller_max_speed_change_m_per_min_s: 0.12,
            puller_max_m_per_min: 0.22,
            source_kp_rpm_per_deg: 0.28,
            source_kd_rpm_per_deg_per_s: 0.22,
            takeup_kp_rpm_per_deg: 0.24,
            takeup_kd_rpm_per_deg_per_s: 0.30,
            source_max_rpm: 7.0,
            takeup_max_rpm: 6.0,
            max_rpm_change_per_s: 5.0,
            takeup_puller_inhibit_deg: 32.0,
            source_high_puller_inhibit_deg: 60.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparePhase {
    Relax,
    Settle,
}

impl RewindControlState {
    pub fn reset_for_prepare(&mut self, now: Instant) {
        self.source_arm.reset();
        self.takeup_arm.reset();
        self.source_follower.force_zero();
        self.takeup_follower.force_zero();
        self.puller_command_m_per_min = 0.0;
        self.puller_accel_m_per_min_s = 0.0;
        self.last_dt_s = 0.0;
        self.last_update = Some(now);
        self.phase_started_at = Some(now);
        self.prepare_settled_since = None;
        self.prepare_phase = PreparePhase::Relax;
        self.prepare_phase_started_at = Some(now);
    }

    pub fn update_prepare_commands(&mut self, now: Instant, dt_s: f64) -> bool {
        let prepare = self.config.prepare;
        let source_error_deg = self.source_arm.filtered_deg - self.config.source_arm.target_deg;
        let takeup_error_deg = self.config.takeup_arm.target_deg - self.takeup_arm.filtered_deg;
        let source_ready =
            prepare_axis_ready(source_error_deg, self.source_arm.rate_deg_per_s, prepare);
        let takeup_ready =
            prepare_axis_ready(takeup_error_deg, self.takeup_arm.rate_deg_per_s, prepare);

        match self.prepare_phase {
            PreparePhase::Relax => {
                self.set_prepare_targets(0.0, 0.0, 0.0, dt_s);
                if self.prepare_phase_elapsed(now) >= prepare.relax_duration {
                    self.set_prepare_phase(PreparePhase::Settle, now);
                }
            }
            PreparePhase::Settle => {
                let source_commands = prepare_source_axis_commands(
                    source_error_deg,
                    self.source_arm.rate_deg_per_s,
                    prepare,
                );
                let takeup_commands = prepare_takeup_axis_commands(
                    takeup_error_deg,
                    self.takeup_arm.rate_deg_per_s,
                    prepare,
                );
                let puller_m_per_min = prepare_puller_command(
                    source_commands.puller_m_per_min,
                    takeup_commands.puller_m_per_min,
                    self.source_arm.filtered_deg,
                    self.takeup_arm.filtered_deg,
                    prepare,
                );
                self.set_prepare_targets(
                    puller_m_per_min,
                    takeup_commands.takeup_rpm,
                    source_commands.source_rpm,
                    dt_s,
                );
            }
        }

        let settled = self.prepare_phase == PreparePhase::Settle
            && source_ready
            && takeup_ready
            && self
                .config
                .source_arm
                .in_start_range(self.source_arm.raw_deg)
            && self
                .config
                .takeup_arm
                .in_start_range(self.takeup_arm.raw_deg);

        if settled {
            let settled_since = self.prepare_settled_since.get_or_insert(now);
            now.duration_since(*settled_since) >= prepare.settle_duration
        } else {
            self.prepare_settled_since = None;
            false
        }
    }

    fn set_prepare_phase(&mut self, phase: PreparePhase, now: Instant) {
        if self.prepare_phase != phase {
            self.prepare_phase = phase;
            self.prepare_phase_started_at = Some(now);
            self.prepare_settled_since = None;
        }
    }

    fn prepare_phase_elapsed(&self, now: Instant) -> Duration {
        self.prepare_phase_started_at
            .map(|started| now.duration_since(started))
            .unwrap_or_default()
    }

    fn set_prepare_targets(
        &mut self,
        puller_m_per_min: f64,
        takeup_rpm: f64,
        source_rpm: f64,
        dt_s: f64,
    ) {
        let prepare = self.config.prepare;
        let previous_puller = self.puller_command_m_per_min;
        self.puller_command_m_per_min = move_toward(
            self.puller_command_m_per_min,
            puller_m_per_min.clamp(0.0, prepare.puller_max_m_per_min),
            prepare.puller_max_speed_change_m_per_min_s,
            dt_s,
        );
        self.puller_accel_m_per_min_s = if dt_s > 0.0 {
            (self.puller_command_m_per_min - previous_puller) / dt_s
        } else {
            0.0
        };

        self.takeup_follower.command_rpm = move_toward(
            self.takeup_follower.command_rpm,
            takeup_rpm.clamp(0.0, prepare.takeup_max_rpm),
            prepare.max_rpm_change_per_s,
            dt_s,
        );
        self.takeup_follower.target_rpm = takeup_rpm;
        self.takeup_follower.trim_rpm = takeup_rpm;
        self.takeup_follower.feed_forward_rpm = 0.0;

        self.source_follower.command_rpm = move_toward(
            self.source_follower.command_rpm,
            source_rpm.clamp(0.0, prepare.source_max_rpm),
            prepare.max_rpm_change_per_s,
            dt_s,
        );
        self.source_follower.target_rpm = source_rpm;
        self.source_follower.trim_rpm = source_rpm;
        self.source_follower.feed_forward_rpm = 0.0;
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct PrepareCommands {
    puller_m_per_min: f64,
    takeup_rpm: f64,
    source_rpm: f64,
}

fn prepare_axis_ready(error_deg: f64, rate_deg_per_s: f64, config: PrepareConfig) -> bool {
    let validation_tolerance_deg = config.settle_tolerance_deg + 4.0;
    let validation_rate_deg_per_s = (config.settle_rate_deg_per_s * 2.5).max(12.0);
    error_deg.abs() <= validation_tolerance_deg && rate_deg_per_s.abs() <= validation_rate_deg_per_s
}

fn prepare_source_axis_commands(
    error_deg: f64,
    rate_deg_per_s: f64,
    config: PrepareConfig,
) -> PrepareCommands {
    if error_deg > config.settle_tolerance_deg {
        let rpm = deadband(error_deg, config.settle_tolerance_deg)
            .mul_add(
                config.source_kp_rpm_per_deg,
                rate_deg_per_s * config.source_kd_rpm_per_deg_per_s,
            )
            .clamp(0.0, config.source_max_rpm);
        PrepareCommands {
            source_rpm: rpm,
            ..Default::default()
        }
    } else if error_deg < -config.settle_tolerance_deg {
        PrepareCommands {
            puller_m_per_min: deadband(-error_deg, config.settle_tolerance_deg)
                .mul_add(config.puller_kp_m_per_min_per_deg, 0.0)
                .clamp(0.0, config.puller_max_m_per_min),
            ..Default::default()
        }
    } else {
        PrepareCommands::default()
    }
}

fn prepare_takeup_axis_commands(
    error_deg: f64,
    rate_deg_per_s: f64,
    config: PrepareConfig,
) -> PrepareCommands {
    if error_deg > config.settle_tolerance_deg {
        let rpm = (deadband(error_deg, config.settle_tolerance_deg) * config.takeup_kp_rpm_per_deg
            - rate_deg_per_s * config.takeup_kd_rpm_per_deg_per_s)
            .clamp(0.0, config.takeup_max_rpm);
        PrepareCommands {
            takeup_rpm: rpm,
            ..Default::default()
        }
    } else if error_deg < -config.settle_tolerance_deg {
        PrepareCommands {
            puller_m_per_min: deadband(-error_deg, config.settle_tolerance_deg)
                .mul_add(config.puller_kp_m_per_min_per_deg, 0.0)
                .clamp(0.0, config.puller_max_m_per_min),
            ..Default::default()
        }
    } else {
        PrepareCommands::default()
    }
}

fn prepare_puller_command(
    source_request_m_per_min: f64,
    takeup_request_m_per_min: f64,
    source_angle_deg: f64,
    takeup_angle_deg: f64,
    config: PrepareConfig,
) -> f64 {
    if takeup_angle_deg < config.takeup_puller_inhibit_deg {
        return 0.0;
    }
    if source_angle_deg > config.source_high_puller_inhibit_deg {
        return 0.0;
    }
    source_request_m_per_min.max(takeup_request_m_per_min)
}

impl Rewinder {
    pub(crate) fn update_prepare_control(&mut self, now: Instant) -> bool {
        if !matches!(self.mode, RewinderMode::Prepare) {
            return false;
        }

        if !self.traverse_controller.is_homed() {
            self.traverse_controller.goto_home();
            self.rewind_control
                .decelerate_motion(self.rewind_control.last_dt_s);
            return true;
        }

        if self.traverse_controller.is_going_home() {
            self.rewind_control
                .decelerate_motion(self.rewind_control.last_dt_s);
            return true;
        }

        if !self.traverse_at_start_position() {
            self.traverse_controller
                .set_target_position(self.traverse_start_position);
            self.traverse_controller.goto_target_position();
            self.rewind_control
                .decelerate_motion(self.rewind_control.last_dt_s);
            return true;
        }

        let Ok((source_angle, takeup_angle)) = self.read_tension_arm_angles_deg() else {
            self.rewind_control.reset_motion();
            return true;
        };

        let dt_s = self
            .rewind_control
            .update_arms(source_angle, takeup_angle, now);
        let prepared = self.rewind_control.update_prepare_commands(now, dt_s);

        if prepared {
            self.rewind_control.decelerate_motion(dt_s);
        }

        true
    }
}
