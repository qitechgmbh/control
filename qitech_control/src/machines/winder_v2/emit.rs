use crate::machines::winder_v2::types::{Mode, SpoolAutomaticActionMode};

use super::{SPOOL_PORT, TRAVERSE_PORT, TraverseMode, WinderV1};

pub use std::time::Instant;

impl<const VARIANT: usize> WinderV1<VARIANT> {
    /// Implement Spool
    /// called by `act`
    pub fn sync_spool_speed(&mut self, t: Instant) {
        let angular_velocity = self.spool_speed_controller.update_speed(
            t,
            &self.tension_arm,
            &self.puller_speed_controller,
        );

        // Apply direction based on forward setting
        let directed_angular_velocity = if self.spool_speed_controller.get_forward() {
            angular_velocity
        } else {
            -angular_velocity
        };

        let steps_per_second = self
            .spool_step_converter
            .angular_velocity_to_steps(directed_angular_velocity);
        let spool_ref = &mut *self.spool.borrow_mut();
        let _ = spool_ref.set_speed(SPOOL_PORT, steps_per_second);
    }

    pub fn stop_or_pull_spool(&mut self, now: Instant) {
        if matches!(
            self.spool_automatic_action.mode.get(),
            SpoolAutomaticActionMode::NoAction
        ) {
            self.calculate_spool_auto_progress_(now);
            return;
        }

        match self.mode.get() {
            Mode::Pull => self.calculate_spool_auto_progress_(now),
            Mode::Wind => self.calculate_spool_auto_progress_(now),
            _ => {
                self.spool_automatic_action.progress_last_check = now;
                return;
            }
        }

        if self.spool_automatic_action.progress >= self.spool_automatic_action.target_length.get() {
            match self.spool_automatic_action.mode.get() {
                SpoolAutomaticActionMode::NoAction => (),
                SpoolAutomaticActionMode::Pull => {
                    self.stop_or_pull_spool_reset(now);
                    self.set_mode(Mode::Pull);
                }
                SpoolAutomaticActionMode::Hold => {
                    self.stop_or_pull_spool_reset(now);
                    self.set_mode(Mode::Hold);
                }
            }
        }
    }
    /// Implement Mode
    pub fn set_mode(&mut self, mode: Mode) {
        let should_update = mode != Mode::Wind || self.can_wind();

        if should_update {
            // all transitions are allowed
            self.mode.set(mode);

            // Apply the mode changes to the spool and puller
            self.set_spool_mode(mode);
            self.set_puller_mode(mode);
            self.set_traverse_mode(mode);
        }
    }

    pub fn traverse_goto_limit_inner(&mut self) {
        if self.traverse_can_goto_limit_inner().is_allowed() {
            self.traverse_controller.goto_limit_inner();
        }
    }

    pub fn traverse_goto_limit_outer(&mut self) {
        if self.traverse_can_goto_limit_outer().is_allowed() {
            self.traverse_controller.goto_limit_outer();
        }
    }

    pub fn traverse_goto_home(&mut self) {
        if self.traverse_can_goto_home().is_allowed() {
            self.traverse_controller.goto_home();
        }
    }

    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_traverse_mode(&mut self, mode: Mode) {
        // Convert to `Winder2Mode` to `TraverseMode`
        let mode: TraverseMode = mode.clone().into();
        // If coming out of standby
        if self.traverse_mode == TraverseMode::Standby && mode != TraverseMode::Standby {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            traverse_ref.set_enabled(TRAVERSE_PORT, true);
            self.traverse_controller.set_enabled(true);
            drop(traverse);
        }

        // If going into standby
        if mode == TraverseMode::Standby && self.traverse_mode != TraverseMode::Standby {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            // If we are going into standby, we need to stop the traverse
            traverse_ref.set_enabled(TRAVERSE_PORT, false);
            self.traverse_controller.set_enabled(false);
            drop(traverse);
        }

        {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            // Transition matrix
            match self.traverse_mode {
                TraverseMode::Standby => match mode {
                    TraverseMode::Standby => {}
                    TraverseMode::Hold => {
                        // From [`TraverseMode::Standby`] to [`TraverseMode::Hold`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, true);
                        self.traverse_controller.set_enabled(true);
                        self.traverse_controller.goto_home();
                    }
                    TraverseMode::Traverse => {
                        // From [`TraverseMode::Standby`] to [`TraverseMode::Wind`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, true);
                        self.traverse_controller.set_enabled(true);
                        self.traverse_controller.start_traversing();
                    }
                },
                TraverseMode::Hold => match mode {
                    TraverseMode::Standby => {
                        // From [`TraverseMode::Hold`] to [`TraverseMode::Standby`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, false);
                        self.traverse_controller.set_enabled(false);
                    }
                    TraverseMode::Hold => {}
                    TraverseMode::Traverse => {
                        // From [`TraverseMode::Hold`] to [`TraverseMode::Wind`]
                        self.traverse_controller.start_traversing();
                    }
                },
                TraverseMode::Traverse => match mode {
                    TraverseMode::Standby => {
                        // From [`TraverseMode::Wind`] to [`TraverseMode::Standby`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, false);
                        self.traverse_controller.set_enabled(false);
                    }
                    TraverseMode::Hold => {
                        // From [`TraverseMode::Wind`] to [`TraverseMode::Hold`]
                        self.traverse_controller.goto_home();
                    }
                    TraverseMode::Traverse => {}
                },
            }
        }

        // Update the internal state
        self.traverse_mode = mode;
    }

    /// Implement Tension Arm
    pub fn tension_arm_zero(&mut self) -> Result<(), String> {
        self.tension_arm.zero()
    }
}
