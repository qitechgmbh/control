use crate::machines::winder2::{
    adaptive_spool_speed_controller::AdaptiveSpoolSpeedController,
    minmax_spool_speed_controller::MinMaxSpoolSpeedController,
    puller_speed_controller::PullerSpeedController,
};
use control_core::controllers::second_degree_motion::acceleration_position_controller::MotionControllerError;

use super::tension_arm::TensionArm;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::info;
use uom::si::f64::AngularVelocity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpoolSpeedControllerType {
    Adaptive,
    MinMax,
}

#[derive(Debug)]
pub struct SpoolSpeedController {
    adaptive_controller: AdaptiveSpoolSpeedController,
    minmax_controller: MinMaxSpoolSpeedController,
    r#type: SpoolSpeedControllerType,
}

impl SpoolSpeedController {
    pub fn new() -> Self {
        Self {
            adaptive_controller: AdaptiveSpoolSpeedController::new(),
            minmax_controller: MinMaxSpoolSpeedController::new(),
            r#type: SpoolSpeedControllerType::MinMax,
        }
    }

    pub fn get_speed(&self) -> AngularVelocity {
        match self.r#type {
            SpoolSpeedControllerType::Adaptive => self.adaptive_controller.get_speed(),
            SpoolSpeedControllerType::MinMax => self.minmax_controller.get_speed(),
        }
    }

    pub fn set_speed(&mut self, speed: AngularVelocity) {
        match self.r#type {
            SpoolSpeedControllerType::Adaptive => self.adaptive_controller.set_speed(speed),
            SpoolSpeedControllerType::MinMax => self.minmax_controller.set_speed(speed),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.adaptive_controller.set_enabled(enabled);
        self.minmax_controller.set_enabled(enabled);
    }

    pub fn is_enabled(&self) -> bool {
        match self.r#type {
            SpoolSpeedControllerType::Adaptive => self.adaptive_controller.is_enabled(),
            SpoolSpeedControllerType::MinMax => self.minmax_controller.is_enabled(),
        }
    }

    pub fn set_type(&mut self, r#type: SpoolSpeedControllerType) {
        // If we're switching to a different type, copy the current speed to the target controller
        if std::mem::discriminant(&self.r#type) != std::mem::discriminant(&r#type) {
            // Get the current speed from the active controller
            let current_speed = match self.r#type {
                SpoolSpeedControllerType::Adaptive => self.adaptive_controller.get_speed(),
                SpoolSpeedControllerType::MinMax => self.minmax_controller.get_speed(),
            };

            // Set the speed in the target controller and reset it for smooth transition
            match r#type {
                SpoolSpeedControllerType::Adaptive => {
                    self.adaptive_controller.set_speed(current_speed);
                    self.adaptive_controller.reset();
                    self.adaptive_controller.set_speed(current_speed); // Set again after reset to maintain speed
                }
                SpoolSpeedControllerType::MinMax => {
                    self.minmax_controller.set_speed(current_speed);
                    self.minmax_controller.reset();
                    self.minmax_controller.set_speed(current_speed); // Set again after reset to maintain speed
                }
            }
        }

        self.r#type = r#type;
    }

    pub fn get_type(&self) -> &SpoolSpeedControllerType {
        &self.r#type
    }

    pub fn set_minmax_min_speed(
        &mut self,
        min_speed: AngularVelocity,
    ) -> Result<(), MotionControllerError> {
        self.minmax_controller.set_min_speed(min_speed)
    }

    pub fn set_minmax_max_speed(
        &mut self,
        max_speed: AngularVelocity,
    ) -> Result<(), MotionControllerError> {
        self.minmax_controller.set_max_speed(max_speed)
    }

    pub fn get_minmax_min_speed(&self) -> AngularVelocity {
        self.minmax_controller.get_min_speed()
    }

    pub fn get_minmax_max_speed(&self) -> AngularVelocity {
        self.minmax_controller.get_max_speed()
    }

    pub fn update_speed(
        &mut self,
        t: Instant,
        tension_arm: &TensionArm,
        puller_speed_controller: &PullerSpeedController,
    ) -> AngularVelocity {
        match self.r#type {
            SpoolSpeedControllerType::Adaptive => {
                self.adaptive_controller
                    .update_speed(t, tension_arm, puller_speed_controller)
            }
            SpoolSpeedControllerType::MinMax => self.minmax_controller.update_speed(t, tension_arm),
        }
    }
}
