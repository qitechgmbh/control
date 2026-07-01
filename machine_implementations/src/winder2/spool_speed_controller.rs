use crate::winder2::{
    adaptive_spool_speed_controller::AdaptiveSpoolSpeedController, clamp_revolution::Clamping,
    minmax_spool_speed_controller::MinMaxSpoolSpeedController,
    puller_speed_controller::PullerSpeedController,
};
use control_core::controllers::second_degree_motion::acceleration_position_controller::MotionControllerError;

use super::tension_arm::TensionArm;
use qitech_lib::units::angle::degree;
use qitech_lib::units::f64::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SpoolSpeedControllerType {
    #[default]
    Adaptive,
    MinMax,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SpoolTensionResponse {
    #[default]
    Takeup,
    Source,
}

impl SpoolTensionResponse {
    pub fn speed_for_clamping(
        self,
        clamping_state: Clamping,
        min_speed: AngularVelocity,
        max_speed: AngularVelocity,
    ) -> Option<AngularVelocity> {
        match (self, clamping_state) {
            (_, Clamping::None) => None,
            (Self::Takeup, Clamping::Min | Clamping::Max) => Some(min_speed),
            (Self::Source, Clamping::Min) => Some(min_speed),
            (Self::Source, Clamping::Max) => Some(max_speed),
        }
    }

    pub fn speed_factor(self, filament_tension: f64) -> f64 {
        match self {
            Self::Takeup => 1.0 - filament_tension,
            Self::Source => filament_tension,
        }
    }

    pub fn speed_factor_change(self, tension_error: f64, proportional_gain: f64) -> f64 {
        match self {
            Self::Takeup => tension_error * proportional_gain,
            Self::Source => -tension_error * proportional_gain,
        }
    }
}

#[derive(Debug)]
pub struct SpoolSpeedController {
    adaptive_controller: AdaptiveSpoolSpeedController,
    minmax_controller: MinMaxSpoolSpeedController,
    r#type: SpoolSpeedControllerType,
    forward: bool,
}

impl Default for SpoolSpeedController {
    fn default() -> Self {
        Self::new()
    }
}

impl SpoolSpeedController {
    pub fn new() -> Self {
        Self::new_with_tension_range(Angle::new::<degree>(90.0), Angle::new::<degree>(20.0))
    }

    pub fn new_with_tension_range(max_angle: Angle, min_angle: Angle) -> Self {
        Self::new_with_tension_range_and_response(
            max_angle,
            min_angle,
            SpoolTensionResponse::Takeup,
        )
    }

    pub fn new_with_tension_range_and_response(
        max_angle: Angle,
        min_angle: Angle,
        tension_response: SpoolTensionResponse,
    ) -> Self {
        Self {
            adaptive_controller: AdaptiveSpoolSpeedController::new_with_tension_range_and_response(
                max_angle,
                min_angle,
                tension_response,
            ),
            minmax_controller: MinMaxSpoolSpeedController::new_with_tension_range_and_response(
                max_angle,
                min_angle,
                tension_response,
            ),
            r#type: SpoolSpeedControllerType::Adaptive,
            forward: true,
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

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.adaptive_controller.set_enabled(enabled);
        self.minmax_controller.set_enabled(enabled);
    }

    pub const fn is_enabled(&self) -> bool {
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

    pub const fn get_type(&self) -> &SpoolSpeedControllerType {
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

    pub fn update_speed_for_angle(
        &mut self,
        t: Instant,
        tension_arm_angle: Angle,
        tension_arm: &TensionArm,
        puller_speed_controller: &PullerSpeedController,
    ) -> AngularVelocity {
        match self.r#type {
            SpoolSpeedControllerType::Adaptive => self.adaptive_controller.update_speed_for_angle(
                t,
                tension_arm_angle,
                puller_speed_controller,
            ),
            SpoolSpeedControllerType::MinMax => self.minmax_controller.update_speed(t, tension_arm),
        }
    }

    // Adaptive controller parameter getters and setters
    pub const fn get_adaptive_tension_target(&self) -> f64 {
        self.adaptive_controller.get_tension_target()
    }

    pub const fn set_adaptive_tension_target(&mut self, tension_target: f64) {
        self.adaptive_controller.set_tension_target(tension_target);
    }

    pub const fn get_adaptive_radius_learning_rate(&self) -> f64 {
        self.adaptive_controller.get_radius_learning_rate()
    }

    pub const fn set_adaptive_radius_learning_rate(&mut self, radius_learning_rate: f64) {
        self.adaptive_controller
            .set_radius_learning_rate(radius_learning_rate);
    }

    pub const fn get_adaptive_max_speed_multiplier(&self) -> f64 {
        self.adaptive_controller.get_max_speed_multiplier()
    }

    pub const fn set_adaptive_max_speed_multiplier(&mut self, max_speed_multiplier: f64) {
        self.adaptive_controller
            .set_max_speed_multiplier(max_speed_multiplier);
    }

    pub const fn get_adaptive_acceleration_factor(&self) -> f64 {
        self.adaptive_controller.get_acceleration_factor()
    }

    pub const fn set_adaptive_acceleration_factor(&mut self, acceleration_factor: f64) {
        self.adaptive_controller
            .set_acceleration_factor(acceleration_factor);
    }

    pub const fn get_adaptive_deacceleration_urgency_multiplier(&self) -> f64 {
        self.adaptive_controller
            .get_deacceleration_urgency_multiplier()
    }

    pub const fn set_adaptive_deacceleration_urgency_multiplier(
        &mut self,
        deacceleration_urgency_multiplier: f64,
    ) {
        self.adaptive_controller
            .set_deacceleration_urgency_multiplier(deacceleration_urgency_multiplier);
    }

    pub const fn get_forward(&self) -> bool {
        self.forward
    }

    pub const fn set_forward(&mut self, forward: bool) {
        self.forward = forward;
    }
}
