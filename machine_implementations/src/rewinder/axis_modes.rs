use super::{
    PULLER_PORT, PullerMode, Rewinder, RewinderMode, SOURCE_SPOOL_PORT, SourceSpoolMode,
    TAKEUP_SPOOL_PORT, TRAVERSE_PORT, TakeupSpoolMode, TraverseMode,
};

impl Rewinder {
    pub(super) fn apply_mode_to_axes(&mut self, mode: &RewinderMode) {
        self.set_takeup_spool_mode(mode);
        self.set_source_spool_mode(mode);
        self.set_puller_mode(mode);
        self.set_traverse_mode(mode);
    }

    /// Apply mode changes to the takeup spool.
    ///
    /// This mirrors Winder2's transition-matrix style: the high-level machine
    /// mode is converted into an axis mode, then only the required hardware
    /// transitions are applied.
    fn set_takeup_spool_mode(&mut self, mode: &RewinderMode) {
        let mode = if matches!(mode, RewinderMode::Hold) && self.hold_decelerating_from_rewind {
            TakeupSpoolMode::Drive
        } else {
            mode.clone().into()
        };
        let spool = &mut *self.takeup_spool.borrow_mut();

        match self.takeup_spool_mode {
            TakeupSpoolMode::Standby => match mode {
                TakeupSpoolMode::Standby => {}
                TakeupSpoolMode::Hold => {
                    spool.set_enabled(TAKEUP_SPOOL_PORT, true);
                }
                TakeupSpoolMode::Drive => {
                    spool.set_enabled(TAKEUP_SPOOL_PORT, true);
                    self.takeup_spool_speed_controller.set_enabled(true);
                }
            },
            TakeupSpoolMode::Hold => match mode {
                TakeupSpoolMode::Standby => {
                    spool.set_enabled(TAKEUP_SPOOL_PORT, false);
                }
                TakeupSpoolMode::Hold => {}
                TakeupSpoolMode::Drive => {
                    self.takeup_spool_speed_controller.set_enabled(true);
                }
            },
            TakeupSpoolMode::Drive => match mode {
                TakeupSpoolMode::Standby => {
                    spool.set_enabled(TAKEUP_SPOOL_PORT, false);
                    self.takeup_spool_speed_controller.set_enabled(false);
                }
                TakeupSpoolMode::Hold => {
                    self.takeup_spool_speed_controller.set_enabled(false);
                }
                TakeupSpoolMode::Drive => {}
            },
        }

        self.takeup_spool_mode = mode;
    }

    /// Apply mode changes to the source spool.
    fn set_source_spool_mode(&mut self, mode: &RewinderMode) {
        let mode = if matches!(mode, RewinderMode::Hold) && self.hold_decelerating_from_rewind {
            SourceSpoolMode::Drive
        } else {
            mode.clone().into()
        };
        let spool = &mut *self.source_spool.borrow_mut();

        match self.source_spool_mode {
            SourceSpoolMode::Standby => match mode {
                SourceSpoolMode::Standby => {}
                SourceSpoolMode::Hold => {
                    spool.set_enabled(SOURCE_SPOOL_PORT, true);
                }
                SourceSpoolMode::Drive => {
                    spool.set_enabled(SOURCE_SPOOL_PORT, true);
                    self.source_spool_speed_controller.set_enabled(true);
                }
            },
            SourceSpoolMode::Hold => match mode {
                SourceSpoolMode::Standby => {
                    spool.set_enabled(SOURCE_SPOOL_PORT, false);
                }
                SourceSpoolMode::Hold => {}
                SourceSpoolMode::Drive => {
                    self.source_spool_speed_controller.set_enabled(true);
                }
            },
            SourceSpoolMode::Drive => match mode {
                SourceSpoolMode::Standby => {
                    spool.set_enabled(SOURCE_SPOOL_PORT, false);
                    self.source_spool_speed_controller.set_enabled(false);
                }
                SourceSpoolMode::Hold => {
                    self.source_spool_speed_controller.set_enabled(false);
                }
                SourceSpoolMode::Drive => {}
            },
        }

        self.source_spool_mode = mode;
    }

    /// Apply mode changes to the puller.
    fn set_puller_mode(&mut self, mode: &RewinderMode) {
        let mode = if matches!(mode, RewinderMode::Hold) && self.hold_decelerating_from_rewind {
            PullerMode::Pull
        } else {
            mode.clone().into()
        };
        let puller = &mut *self.puller.borrow_mut();

        match self.puller_mode {
            PullerMode::Standby => match mode {
                PullerMode::Standby => {}
                PullerMode::Hold => {
                    puller.set_enabled(PULLER_PORT, true);
                }
                PullerMode::Pull => {
                    puller.set_enabled(PULLER_PORT, true);
                    self.puller_speed_controller.set_enabled(true);
                }
            },
            PullerMode::Hold => match mode {
                PullerMode::Standby => {
                    puller.set_enabled(PULLER_PORT, false);
                }
                PullerMode::Hold => {}
                PullerMode::Pull => {
                    self.puller_speed_controller.set_enabled(true);
                }
            },
            PullerMode::Pull => match mode {
                PullerMode::Standby => {
                    puller.set_enabled(PULLER_PORT, false);
                    self.puller_speed_controller.set_enabled(false);
                }
                PullerMode::Hold => {
                    self.puller_speed_controller.set_enabled(false);
                }
                PullerMode::Pull => {}
            },
        }

        self.puller_mode = mode;
    }

    /// Apply mode changes to the traverse.
    fn set_traverse_mode(&mut self, mode: &RewinderMode) {
        let mode: TraverseMode = mode.clone().into();
        let traverse = &mut *self.traverse.borrow_mut();
        match self.traverse_mode {
            TraverseMode::Standby => match mode {
                TraverseMode::Standby => {}
                TraverseMode::Hold => {
                    traverse.set_enabled(TRAVERSE_PORT, true);
                    self.traverse_controller.set_enabled(true);
                }
                TraverseMode::Traverse => {
                    traverse.set_enabled(TRAVERSE_PORT, true);
                    self.traverse_controller.set_enabled(true);
                    self.traverse_controller.start_traversing();
                }
            },
            TraverseMode::Hold => match mode {
                TraverseMode::Standby => {
                    traverse.set_enabled(TRAVERSE_PORT, false);
                    self.traverse_controller.set_enabled(false);
                }
                TraverseMode::Hold => {}
                TraverseMode::Traverse => {
                    self.traverse_controller.start_traversing();
                }
            },
            TraverseMode::Traverse => match mode {
                TraverseMode::Standby => {
                    traverse.set_enabled(TRAVERSE_PORT, false);
                    self.traverse_controller.set_enabled(false);
                }
                TraverseMode::Hold => {
                    self.traverse_controller.stop();
                }
                TraverseMode::Traverse => {}
            },
        }

        self.traverse_mode = mode;
    }
}
