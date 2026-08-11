use crate::machines::winder_v2::{
    adaptive_spool_speed_controller::AdaptiveSpoolSpeedController,
    minmax_spool_speed_controller::MinMaxSpoolSpeedController,
    puller_speed_controller::PullerSpeedController,
};

use super::tension_arm::TensionArm;
use qitech_framework::EnumProperty;
use qitech_framework::machine::ConfigProperty;
use qitech_lib::units::f64::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default, EnumProperty)]
pub enum SpoolSpeedControllerType {
    #[default]
    Adaptive,
    MinMax,
}

pub struct SpoolSpeedController {
    pub adaptive_controller: AdaptiveSpoolSpeedController,
    pub min_max_controller: MinMaxSpoolSpeedController,
    r#type: ConfigProperty<SpoolSpeedControllerType>,
    forward: ConfigProperty<bool>,
}

impl SpoolSpeedController {
    pub fn new(
        r#type: ConfigProperty<SpoolSpeedControllerType>,
        forward: ConfigProperty<bool>,
        min_max_controller: MinMaxSpoolSpeedController,
        adaptive_controller: AdaptiveSpoolSpeedController,
    ) -> Self {
        Self {
            r#type,
            forward,
            adaptive_controller,
            min_max_controller,
        }
    }

    pub fn get_speed(&self) -> AngularVelocity {
        match self.r#type.get() {
            SpoolSpeedControllerType::Adaptive => self.adaptive_controller.get_speed(),
            SpoolSpeedControllerType::MinMax => self.min_max_controller.get_speed(),
        }
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.adaptive_controller.set_enabled(enabled);
        self.min_max_controller.set_enabled(enabled);
    }

    pub fn on_control_mode_changed(&mut self) {
        // Get the current speed from the active controller
        let current_speed = match self.r#type.get() {
            SpoolSpeedControllerType::Adaptive => self.adaptive_controller.get_speed(),
            SpoolSpeedControllerType::MinMax => self.min_max_controller.get_speed(),
        };

        // Set the speed in the target controller and reset it for smooth transition
        match self.r#type.get() {
            SpoolSpeedControllerType::Adaptive => {
                self.adaptive_controller.set_speed(current_speed);
                self.adaptive_controller.reset();
                self.adaptive_controller.set_speed(current_speed); // Set again after reset to maintain speed
            }
            SpoolSpeedControllerType::MinMax => {
                self.min_max_controller.set_speed(current_speed);
                self.min_max_controller.reset();
                self.min_max_controller.set_speed(current_speed); // Set again after reset to maintain speed
            }
        }
    }

    pub fn update_speed(
        &mut self,
        t: Instant,
        tension_arm: &TensionArm,
        puller_speed_controller: &PullerSpeedController,
    ) -> AngularVelocity {
        match self.r#type.get() {
            SpoolSpeedControllerType::Adaptive => {
                self.adaptive_controller
                    .update_speed(t, tension_arm, puller_speed_controller)
            }
            SpoolSpeedControllerType::MinMax => {
                self.min_max_controller.update_speed(t, tension_arm)
            }
        }
    }

    pub fn get_forward(&self) -> bool {
        self.forward.get()
    }
}
