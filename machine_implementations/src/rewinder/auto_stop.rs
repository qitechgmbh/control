use super::{RewindPhase, Rewinder, RewinderMode, api::RewindAutomaticActionMode};
use qitech_lib::units::{ConstZero, Length, length::meter, velocity::meter_per_second};
use std::time::Instant;

#[derive(Debug)]
pub struct RewindAutomaticAction {
    pub progress: Length,
    progress_last_check: Instant,
    pub target_length: Length,
    pub mode: RewindAutomaticActionMode,
}

impl Default for RewindAutomaticAction {
    fn default() -> Self {
        Self {
            progress: Length::ZERO,
            progress_last_check: Instant::now(),
            target_length: Length::ZERO,
            mode: RewindAutomaticActionMode::default(),
        }
    }
}

impl Rewinder {
    pub fn reset_rewind_progress(&mut self, now: Instant) {
        self.rewind_automatic_action.progress = Length::ZERO;
        self.rewind_automatic_action.progress_last_check = now;
    }

    pub fn update_rewind_progress(&mut self, now: Instant) {
        let dt_s = now
            .duration_since(self.rewind_automatic_action.progress_last_check)
            .as_secs_f64();
        let meters_this_interval = Length::new::<meter>(
            self.puller_speed_controller
                .last_speed
                .get::<meter_per_second>()
                * dt_s,
        );
        self.rewind_automatic_action.progress += meters_this_interval.abs();
        self.rewind_automatic_action.progress_last_check = now;
    }

    pub fn stop_or_pull_rewind(&mut self, now: Instant) {
        let can_progress = matches!(self.mode, RewinderMode::Pull)
            || (matches!(self.mode, RewinderMode::Rewind)
                && matches!(
                    self.rewind_phase,
                    RewindPhase::CrawlStart | RewindPhase::Rewind
                ));

        if can_progress {
            self.update_rewind_progress(now);
        } else {
            self.rewind_automatic_action.progress_last_check = now;
            return;
        }

        if matches!(
            self.rewind_automatic_action.mode,
            RewindAutomaticActionMode::NoAction
        ) {
            return;
        }

        if self.rewind_automatic_action.target_length.get::<meter>() <= 0.0 {
            return;
        }

        if self.rewind_automatic_action.progress >= self.rewind_automatic_action.target_length {
            match self.rewind_automatic_action.mode {
                RewindAutomaticActionMode::NoAction => {}
                RewindAutomaticActionMode::Hold => {
                    self.reset_rewind_progress(now);
                    self.set_mode(&RewinderMode::Hold);
                }
            }
        }
    }

    pub fn set_rewind_automatic_required_meters(&mut self, meters: f64) {
        self.rewind_automatic_action.target_length = Length::new::<meter>(meters.max(0.0));
        self.emit_state();
    }

    pub fn set_rewind_automatic_action(&mut self, mode: RewindAutomaticActionMode) {
        self.rewind_automatic_action.mode = mode;
        self.emit_state();
    }
}
