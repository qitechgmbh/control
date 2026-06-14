use super::{RewindPhase, Rewinder, RewinderMode, DIAGNOSTIC_LOGS_ENABLED};
use qitech_lib::units::{angular_velocity::revolution_per_minute, velocity::meter_per_minute};
use std::time::Instant;

impl Rewinder {
    pub fn log_rewind_diagnostics(&mut self, now: Instant) {
        if !DIAGNOSTIC_LOGS_ENABLED {
            return;
        }
        if !matches!(self.mode, RewinderMode::Rewind | RewinderMode::Prepare) {
            return;
        }
        if now
            .duration_since(self.last_rewind_diagnostics_log)
            .as_secs_f64()
            < 0.5
        {
            return;
        }
        self.last_rewind_diagnostics_log = now;

        let live_values = self.get_live_values();
        if matches!(self.mode, RewinderMode::Prepare) {
            println!(
                "Rewinder prepare: phase={:?} puller_command={:.2}m/min puller_actual={:.2}m/min takeup_angle={:.1}deg takeup_filtered={:.1}deg takeup_rate={:.1}deg/s takeup_cmd={:.1}rpm source_angle={:.1}deg source_filtered={:.1}deg source_rate={:.1}deg/s source_cmd={:.1}rpm",
                self.rewind_control.prepare_phase,
                self.rewind_control
                    .puller_command_speed()
                    .get::<meter_per_minute>(),
                live_values.puller_speed,
                live_values.takeup_tension_arm_angle,
                self.rewind_control.takeup_arm.filtered_deg,
                self.rewind_control.takeup_arm.rate_deg_per_s,
                self.rewind_control
                    .takeup_command_angular_velocity()
                    .get::<revolution_per_minute>(),
                live_values.source_tension_arm_angle,
                self.rewind_control.source_arm.filtered_deg,
                self.rewind_control.source_arm.rate_deg_per_s,
                self.rewind_control
                    .source_command_angular_velocity()
                    .get::<revolution_per_minute>(),
            );
            return;
        }

        println!(
            "Rewinder diag: phase={:?} puller_target={:.2}m/min puller_command={:.2}m/min puller_accel={:.2}m/min/s puller_actual={:.2}m/min takeup_angle={:.1}deg takeup_filtered={:.1}deg takeup_zone={:?} takeup_rate={:.1}deg/s takeup_controller={:.1}rpm source_angle={:.1}deg source_filtered={:.1}deg source_zone={:?} source_rate={:.1}deg/s source_recovery={} source_ff={:.1}rpm source_trim={:.1}rpm source_target={:.1}rpm source_cmd={:.1}rpm source_ratio={:.2} takeup_actual={:.1}rpm source_actual={:.1}rpm can_rewind={} reason={}",
            self.rewind_phase,
            self.puller_speed_controller
                .get_target_speed()
                .get::<meter_per_minute>(),
            self.rewind_control
                .puller_command_speed()
                .get::<meter_per_minute>(),
            self.rewind_control.puller_accel_m_per_min_s,
            live_values.puller_speed,
            live_values.takeup_tension_arm_angle,
            self.rewind_control.takeup_arm.filtered_deg,
            self.rewind_control.takeup_arm.zone,
            self.rewind_control.takeup_arm.rate_deg_per_s,
            self.takeup_spool_speed_controller
                .get_speed()
                .get::<revolution_per_minute>(),
            live_values.source_tension_arm_angle,
            self.rewind_control.source_arm.filtered_deg,
            self.rewind_control.source_arm.zone,
            self.rewind_control.source_arm.rate_deg_per_s,
            self.rewind_control.source_recovery_active(),
            self.rewind_control.source_follower.feed_forward_rpm,
            self.rewind_control.source_follower.trim_rpm,
            self.rewind_control.source_follower.target_rpm,
            self.rewind_control
                .source_command_angular_velocity()
                .get::<revolution_per_minute>(),
            self.rewind_control.source_follower.ratio_rpm_per_m_per_min,
            live_values.takeup_spool_rpm,
            live_values.source_spool_rpm,
            !matches!(self.rewind_phase, RewindPhase::FaultHold),
            if matches!(self.rewind_phase, RewindPhase::FaultHold) {
                self.rewind_hard_stop_reason
                    .as_deref()
                    .unwrap_or("rewind hard stop")
            } else {
                "ok"
            }
        );
    }
}
