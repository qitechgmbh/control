use super::Rewinder;
use qitech_lib::units::{angle::degree, f64::*};

impl Rewinder {
    pub(crate) fn normalize_tension_arm_angle_deg(angle: Angle) -> f64 {
        let angle_deg = angle.get::<degree>();
        if angle_deg >= 270.0 {
            angle_deg - 360.0
        } else {
            angle_deg
        }
    }

    pub(crate) fn read_tension_arm_angles_deg(&self) -> Result<(f64, f64), &'static str> {
        let source_angle = self
            .source_tension_arm
            .get_angle()
            .map(Self::normalize_tension_arm_angle_deg)
            .map_err(|_| "failed to read source tension arm angle")?;
        let takeup_angle = self
            .takeup_tension_arm
            .get_angle()
            .map(Self::normalize_tension_arm_angle_deg)
            .map_err(|_| "failed to read takeup tension arm angle")?;
        Ok((source_angle, takeup_angle))
    }

    pub fn can_rewind(&self) -> bool {
        self.rewind_block_reason().is_none()
    }

    pub fn rewind_block_reason(&self) -> Option<&'static str> {
        self.rewind_block_reason_with_start_window(true)
    }

    pub fn active_rewind_block_reason(&self) -> Option<&'static str> {
        self.rewind_block_reason_with_start_window(false)
    }

    pub fn prepare_block_reason(&self) -> Option<&'static str> {
        if !self.takeup_tension_arm.zeroed {
            Some("takeup tension arm is not zeroed")
        } else if !self.source_tension_arm.zeroed {
            Some("source tension arm is not zeroed")
        } else if self
            .source_tension_arm
            .get_angle()
            .map(Self::normalize_tension_arm_angle_deg)
            .map_or(true, |angle| !(-45.0..=135.0).contains(&angle))
        {
            Some("source tension arm sensor is outside prepare recovery range")
        } else if self
            .takeup_tension_arm
            .get_angle()
            .map(Self::normalize_tension_arm_angle_deg)
            .map_or(true, |angle| !(-45.0..=135.0).contains(&angle))
        {
            Some("takeup tension arm sensor is outside prepare recovery range")
        } else {
            None
        }
    }

    fn rewind_block_reason_with_start_window(
        &self,
        enforce_start_window: bool,
    ) -> Option<&'static str> {
        let source_config = self.rewind_control.config.source_arm;
        let takeup_config = self.rewind_control.config.takeup_arm;
        if !self.takeup_tension_arm.zeroed {
            Some("takeup tension arm is not zeroed")
        } else if !self.source_tension_arm.zeroed {
            Some("source tension arm is not zeroed")
        } else if !self.traverse_controller.is_homed() {
            Some("traverse is not homed")
        } else if self.traverse_controller.is_going_home() {
            Some("traverse is still homing")
        } else if enforce_start_window
            && self
                .source_tension_arm
                .get_angle()
                .map(Self::normalize_tension_arm_angle_deg)
                .map_or(true, |angle| !source_config.in_start_range(angle))
        {
            Some("source tension arm is outside rewind start range")
        } else if enforce_start_window
            && self
                .takeup_tension_arm
                .get_angle()
                .map(Self::normalize_tension_arm_angle_deg)
                .map_or(true, |angle| !takeup_config.in_start_range(angle))
        {
            Some("takeup tension arm is outside rewind start range")
        } else if self
            .source_tension_arm
            .get_angle()
            .map(Self::normalize_tension_arm_angle_deg)
            .map_or(true, |angle| !source_config.in_hard_range(angle))
        {
            Some("source tension arm is outside rewind range")
        } else if self
            .takeup_tension_arm
            .get_angle()
            .map(Self::normalize_tension_arm_angle_deg)
            .map_or(true, |angle| !takeup_config.in_hard_range(angle))
        {
            Some("takeup tension arm is outside rewind range")
        } else {
            None
        }
    }
}
