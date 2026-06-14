use super::{RewindPhase, Rewinder, RewinderMode};
use qitech_lib::units::{f64::*, velocity::meter_per_minute};
use std::time::Instant;

impl Rewinder {
    fn set_rewind_phase(&mut self, phase: RewindPhase, reason: &str) {
        if self.rewind_phase != phase {
            println!(
                "Rewinder phase {:?} -> {:?}: {}",
                self.rewind_phase, phase, reason
            );
            self.rewind_control.start_phase(Instant::now());
        }
        if matches!(phase, RewindPhase::FaultHold) {
            self.rewind_hard_stop_reason = Some(reason.to_owned());
        } else {
            self.rewind_hard_stop_reason = None;
        }
        self.rewind_phase = phase;
    }

    pub(crate) fn update_rewind_sequence(&mut self, now: Instant) {
        if !matches!(self.mode, RewinderMode::Rewind) {
            if !matches!(self.rewind_phase, RewindPhase::Idle) {
                self.set_rewind_phase(RewindPhase::Idle, "mode is not Rewind");
            }
            self.rewind_control.reset_motion();
            self.rewind_puller_command_speed = None;
            return;
        }

        if matches!(self.rewind_phase, RewindPhase::FaultHold) {
            self.rewind_control.reset_motion();
            self.rewind_puller_command_speed = Some(Velocity::new::<meter_per_minute>(0.0));
            return;
        }

        if let Some(reason) = self.active_rewind_block_reason() {
            self.set_rewind_phase(RewindPhase::FaultHold, reason);
            self.rewind_control.reset_motion();
            self.rewind_puller_command_speed = Some(Velocity::new::<meter_per_minute>(0.0));
            return;
        }

        let Ok((source_angle, takeup_angle)) = self.read_tension_arm_angles_deg() else {
            self.set_rewind_phase(RewindPhase::FaultHold, "failed to read tension arm angle");
            self.rewind_control.reset_motion();
            self.rewind_puller_command_speed = Some(Velocity::new::<meter_per_minute>(0.0));
            return;
        };

        let dt_s = self
            .rewind_control
            .update_arms(source_angle, takeup_angle, now)
            .max(0.0);

        if self.rewind_control.source_arm.zone.is_fault() {
            self.set_rewind_phase(
                RewindPhase::FaultHold,
                "source tension arm is outside rewind range",
            );
        } else if self.rewind_control.takeup_arm.zone.is_fault() {
            self.set_rewind_phase(
                RewindPhase::FaultHold,
                "takeup tension arm is outside rewind range",
            );
        }

        if matches!(self.rewind_phase, RewindPhase::FaultHold) {
            self.rewind_control.reset_motion();
            self.rewind_puller_command_speed = Some(Velocity::new::<meter_per_minute>(0.0));
            return;
        }

        match self.rewind_phase {
            RewindPhase::Idle => self.set_rewind_phase(RewindPhase::Validate, "rewind requested"),
            RewindPhase::Validate => {
                let source_ok = self
                    .rewind_control
                    .config
                    .source_arm
                    .in_start_range(source_angle);
                let takeup_ok = self
                    .rewind_control
                    .config
                    .takeup_arm
                    .in_start_range(takeup_angle);
                if source_ok && takeup_ok {
                    self.set_rewind_phase(RewindPhase::Precharge, "start angles validated");
                }
            }
            RewindPhase::Precharge => {
                if self.rewind_control.phase_elapsed(now)
                    >= self.rewind_control.config.precharge_duration
                {
                    self.set_rewind_phase(RewindPhase::CrawlStart, "precharge settled");
                }
            }
            RewindPhase::CrawlStart => {
                if self.rewind_control.phase_elapsed(now)
                    >= self.rewind_control.config.crawl_duration
                {
                    self.set_rewind_phase(RewindPhase::Rewind, "crawl start complete");
                }
            }
            RewindPhase::Rewind | RewindPhase::FaultHold => {}
        }

        let ui_target_m_per_min = self
            .puller_speed_controller
            .get_target_speed()
            .get::<meter_per_minute>();
        let commanded_target_m_per_min = match self.rewind_phase {
            RewindPhase::Precharge | RewindPhase::Validate | RewindPhase::Idle => 0.0,
            RewindPhase::CrawlStart => ui_target_m_per_min
                .min(self.rewind_control.config.puller_ramp.crawl_speed_m_per_min),
            RewindPhase::Rewind => ui_target_m_per_min,
            RewindPhase::FaultHold => 0.0,
        };

        self.rewind_control.update_puller_command(
            Velocity::new::<meter_per_minute>(commanded_target_m_per_min),
            dt_s,
        );
        self.rewind_puller_command_speed = Some(self.rewind_control.puller_command_speed());
    }
}
