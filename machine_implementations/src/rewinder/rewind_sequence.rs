use super::{RewindPhase, Rewinder, RewinderMode, api::HardStopEvent};
use qitech_lib::units::{
    angular_velocity::revolution_per_minute, f64::*, velocity::meter_per_minute,
};
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
        self.rewind_phase = phase;
    }

    fn hard_stop_to_standby(
        &mut self,
        reason: String,
        source_angle: Option<f64>,
        takeup_angle: Option<f64>,
    ) {
        let source_config = self.rewind_control.config.source_arm;
        let takeup_config = self.rewind_control.config.takeup_arm;
        let source_out_of_range = source_angle
            .map(|angle| !source_config.in_hard_range(angle))
            .unwrap_or(false);
        let takeup_out_of_range = takeup_angle
            .map(|angle| !takeup_config.in_hard_range(angle))
            .unwrap_or(false);

        tracing::warn!("Rewinder hard stop: {reason}");

        self.rewind_control.reset_motion();
        self.puller_speed_controller
            .reset_speed(Velocity::new::<meter_per_minute>(0.0));
        self.takeup_spool_speed_controller
            .set_speed(AngularVelocity::new::<revolution_per_minute>(0.0));
        self.source_spool_speed_controller
            .set_speed(AngularVelocity::new::<revolution_per_minute>(0.0));

        self.emit_hard_stop(HardStopEvent {
            reason,
            source_angle,
            takeup_angle,
            source_min_angle: source_config.hard_min_deg,
            source_max_angle: source_config.hard_max_deg,
            takeup_min_angle: takeup_config.hard_min_deg,
            takeup_max_angle: takeup_config.hard_max_deg,
            source_out_of_range,
            takeup_out_of_range,
        });

        self.mode = RewinderMode::Standby;
        self.rewind_phase = RewindPhase::Idle;
        self.apply_mode_to_axes(&RewinderMode::Standby);
        self.emit_state();
    }

    fn rewind_hard_stop_reason(
        &self,
        source_out_of_range: bool,
        takeup_out_of_range: bool,
    ) -> String {
        match (source_out_of_range, takeup_out_of_range) {
            (true, true) => "source and takeup tension arms are outside rewind range".to_owned(),
            (true, false) => "source tension arm is outside rewind range".to_owned(),
            (false, true) => "takeup tension arm is outside rewind range".to_owned(),
            (false, false) => "runtime hard stop".to_owned(),
        }
    }

    pub(crate) fn update_rewind_sequence(&mut self, now: Instant) {
        if !matches!(self.mode, RewinderMode::Rewind) {
            if !matches!(self.rewind_phase, RewindPhase::Idle) {
                self.set_rewind_phase(RewindPhase::Idle, "mode is not Rewind");
            }
            if !matches!(self.mode, RewinderMode::Hold) {
                self.rewind_control.reset_motion();
            }
            return;
        }

        if let Some(reason) = self.active_rewind_block_reason() {
            let angles = self.read_tension_arm_angles_deg().ok();
            self.hard_stop_to_standby(
                reason.to_owned(),
                angles.map(|(source_angle, _)| source_angle),
                angles.map(|(_, takeup_angle)| takeup_angle),
            );
            return;
        }

        let Ok((source_angle, takeup_angle)) = self.read_tension_arm_angles_deg() else {
            self.hard_stop_to_standby("failed to read tension arm angle".to_owned(), None, None);
            return;
        };

        let dt_s = self
            .rewind_control
            .update_arms(source_angle, takeup_angle, now)
            .max(0.0);

        let source_fault = self.rewind_control.source_arm.zone.is_fault();
        let takeup_fault = self.rewind_control.takeup_arm.zone.is_fault();
        if source_fault || takeup_fault {
            self.hard_stop_to_standby(
                self.rewind_hard_stop_reason(source_fault, takeup_fault),
                Some(source_angle),
                Some(takeup_angle),
            );
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
            RewindPhase::Rewind => {}
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
        };

        self.rewind_control.update_puller_command(
            Velocity::new::<meter_per_minute>(commanded_target_m_per_min),
            dt_s,
        );
    }
}
