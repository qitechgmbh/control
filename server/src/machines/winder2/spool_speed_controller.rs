use crate::machines::winder2::{
    adaptive_spool_speed_controller::AdaptiveSpoolSpeedController,
    minmax_spool_speed_controller::MinMaxSpoolSpeedController,
    puller_speed_controller::PullerSpeedController,
};
use control_core::{
    controllers::second_degree_motion::acceleration_position_controller::MotionControllerError,
    helpers::moving_time_window::MovingTimeWindow,
};

use super::tension_arm::TensionArm;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uom::si::{
    f64::{AngularVelocity, Length},
    length::meter,
};

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
    radius_history: MovingTimeWindow<f64>,
}

impl SpoolSpeedController {
    pub fn new() -> Self {
        Self {
            adaptive_controller: AdaptiveSpoolSpeedController::new(),
            minmax_controller: MinMaxSpoolSpeedController::new(),
            radius_history: MovingTimeWindow::new(Duration::from_secs(10), 40), // Example size, adjust as needed
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
        let angular_velocity = match self.r#type {
            SpoolSpeedControllerType::Adaptive => {
                self.adaptive_controller
                    .update_speed(t, tension_arm, puller_speed_controller)
            }
            SpoolSpeedControllerType::MinMax => self.minmax_controller.update_speed(t, tension_arm),
        };

        {
            let radius = match self.r#type {
                SpoolSpeedControllerType::Adaptive => self.adaptive_controller.get_radius(),
                SpoolSpeedControllerType::MinMax => {
                    self.minmax_controller.get_radius(puller_speed_controller)
                }
            };

            // add radius to history
            self.radius_history.update(radius.get::<meter>(), t);
        }

        angular_velocity
    }

    pub fn get_estimated_radius(&mut self) -> Length {
        // Use the average of the last 10 radius measurements
        Length::new::<meter>(self.radius_history.average())
    }

    // Adaptive controller parameter getters and setters
    pub fn get_adaptive_tension_target(&self) -> f64 {
        self.adaptive_controller.get_tension_target()
    }

    pub fn set_adaptive_tension_target(&mut self, tension_target: f64) {
        self.adaptive_controller.set_tension_target(tension_target);
    }

    pub fn get_adaptive_radius_learning_rate(&self) -> f64 {
        self.adaptive_controller.get_radius_learning_rate()
    }

    pub fn set_adaptive_radius_learning_rate(&mut self, radius_learning_rate: f64) {
        self.adaptive_controller
            .set_radius_learning_rate(radius_learning_rate);
    }

    pub fn get_adaptive_max_speed_multiplier(&self) -> f64 {
        self.adaptive_controller.get_max_speed_multiplier()
    }

    pub fn set_adaptive_max_speed_multiplier(&mut self, max_speed_multiplier: f64) {
        self.adaptive_controller
            .set_max_speed_multiplier(max_speed_multiplier);
    }

    pub fn get_adaptive_acceleration_factor(&self) -> f64 {
        self.adaptive_controller.get_acceleration_factor()
    }

    pub fn set_adaptive_acceleration_factor(&mut self, acceleration_factor: f64) {
        self.adaptive_controller
            .set_acceleration_factor(acceleration_factor);
    }

    pub fn get_adaptive_deacceleration_urgency_multiplier(&self) -> f64 {
        self.adaptive_controller
            .get_deacceleration_urgency_multiplier()
    }

    pub fn set_adaptive_deacceleration_urgency_multiplier(
        &mut self,
        deacceleration_urgency_multiplier: f64,
    ) {
        self.adaptive_controller
            .set_deacceleration_urgency_multiplier(deacceleration_urgency_multiplier);
    }
}
