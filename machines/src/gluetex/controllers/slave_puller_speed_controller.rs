use std::time::Instant;

use control_core::converters::linear_step_converter::LinearStepConverter;
use units::ConstZero;
use units::f64::*;

use super::tension_arm::TensionArm;
use crate::gluetex::features::filament_tension::FilamentTensionCalculator;

/// Slave puller speed controller that follows the master puller
/// with speed adjustments based on tension arm angle feedback.
///
/// Uses min/max angles to define the detection zone for speed control.
/// When tension arm is at min angle (low tension), slave pulls faster.
/// When tension arm is at max angle (high tension), slave pulls slower.
/// Optional min/max speed factors provide overspeed protection.
#[derive(Debug)]
pub struct SlavePullerSpeedController {
    /// Last commanded velocity
    last_speed: Velocity,
    /// Whether the controller is enabled
    enabled: bool,
    /// Forward rotation direction
    forward: bool,
    /// Target angle for the tension arm (setpoint)
    target_angle: Angle,
    /// Sensitivity range around target angle (in degrees) for speed adjustment
    sensitivity: Angle,
    /// Optional minimum speed factor for overspeed protection (e.g., 0.5 = 50%)
    min_speed_factor: Option<f64>,
    /// Optional maximum speed factor for overspeed protection (e.g., 2.0 = 200%)
    max_speed_factor: Option<f64>,
    /// Tension calculator to convert arm angle to normalized tension
    filament_calc: FilamentTensionCalculator,
    /// Converter for linear velocity to motor steps
    pub converter: LinearStepConverter,
}

impl SlavePullerSpeedController {
    /// Create a new slave puller speed controller
    ///
    /// # Arguments
    /// * `target_angle` - Target angle for the tension arm (setpoint)
    /// * `sensitivity` - Sensitivity range around target for speed adjustment (degrees)
    /// * `converter` - Linear step converter for the motor
    /// * `filament_calc` - Tension calculator for the tension arm
    pub fn new(
        target_angle: Angle,
        sensitivity: Angle,
        converter: LinearStepConverter,
        filament_calc: FilamentTensionCalculator,
    ) -> Self {
        Self {
            last_speed: Velocity::ZERO,
            enabled: false,
            forward: true,
            target_angle,
            sensitivity,
            min_speed_factor: None,
            max_speed_factor: None,
            filament_calc,
            converter,
        }
    }

    /// Calculate the raw target speed based on master speed and tension arm angle
    ///
    /// # Arguments
    /// * `master_speed` - Current speed of the master puller
    /// * `tension_arm` - Reference to the tension arm for feedback
    ///
    /// # Returns
    /// Target velocity for the slave puller
    fn speed_raw(&self, master_speed: Velocity, tension_arm: &TensionArm) -> Velocity {
        // Get tension arm angle
        let current_angle = tension_arm.get_angle();

        // Calculate deviation from target angle
        let deviation = current_angle - self.target_angle;

        // Normalize deviation to [-1.0, 1.0] based on sensitivity range
        // Positive deviation = angle above target = speed up
        // Negative deviation = angle below target = slow down
        let normalized_deviation = (deviation.get::<units::angle::degree>()
            / self.sensitivity.get::<units::angle::degree>())
        .clamp(-1.0, 1.0);

        // Calculate speed factor:
        // -1.0 (below target) -> 0.5 (50% speed - slow down)
        //  0.0 (at target)    -> 1.0 (100% speed)
        // +1.0 (above target) -> 1.5 (150% speed - speed up)
        let speed_factor = 1.0 + (normalized_deviation * 0.5);

        let base_speed = master_speed * speed_factor;

        // Apply optional speed factor limits for overspeed protection
        let limited_speed = if let Some(min_factor) = self.min_speed_factor {
            base_speed.max(master_speed * min_factor)
        } else {
            base_speed
        };

        let limited_speed = if let Some(max_factor) = self.max_speed_factor {
            limited_speed.min(master_speed * max_factor)
        } else {
            limited_speed
        };

        limited_speed.abs()
    }

    /// Main update function called every control cycle
    ///
    /// # Arguments
    /// * `t` - Current timestamp
    /// * `master_speed` - Speed of the master puller
    /// * `tension_arm` - Reference to the slave tension arm
    ///
    /// # Returns
    /// Commanded velocity for the slave puller motor
    pub fn update_speed(
        &mut self,
        _t: Instant,
        master_speed: Velocity,
        tension_arm: &TensionArm,
    ) -> Velocity {
        // Calculate raw target speed
        let target_speed = self.speed_raw(master_speed, tension_arm);

        // Apply enable/disable logic
        let final_speed = if self.enabled {
            target_speed
        } else {
            Velocity::ZERO
        };

        // Store and return final speed
        self.last_speed = final_speed;
        final_speed
    }

    // Getters and setters

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub const fn set_forward(&mut self, forward: bool) {
        self.forward = forward;
    }

    pub const fn get_forward(&self) -> bool {
        self.forward
    }

    pub const fn set_target_angle(&mut self, angle: Angle) {
        self.target_angle = angle;
    }

    pub const fn get_target_angle(&self) -> Angle {
        self.target_angle
    }

    pub const fn set_sensitivity(&mut self, sensitivity: Angle) {
        self.sensitivity = sensitivity;
    }

    pub const fn get_sensitivity(&self) -> Angle {
        self.sensitivity
    }

    pub fn set_min_speed_factor(&mut self, factor: Option<f64>) {
        self.min_speed_factor = factor.map(|f| f.clamp(0.1, 5.0));
    }

    pub const fn get_min_speed_factor(&self) -> Option<f64> {
        self.min_speed_factor
    }

    pub fn set_max_speed_factor(&mut self, factor: Option<f64>) {
        self.max_speed_factor = factor.map(|f| f.clamp(0.1, 5.0));
    }

    pub const fn get_max_speed_factor(&self) -> Option<f64> {
        self.max_speed_factor
    }

    pub const fn get_last_speed(&self) -> Velocity {
        self.last_speed
    }

    pub fn reset(&mut self) {
        self.last_speed = Velocity::ZERO;
    }

    /// Convert velocity to angular velocity for step conversion
    pub fn velocity_to_angular_velocity(&self, velocity: Velocity) -> AngularVelocity {
        self.converter.velocity_to_angular_velocity(velocity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use units::angle::degree;
    use units::length::millimeter;

    #[test]
    fn test_slave_puller_angles() {
        let converter =
            LinearStepConverter::from_circumference(200, Length::new::<millimeter>(50.0));
        let filament_calc =
            FilamentTensionCalculator::new(Angle::new::<degree>(20.0), Angle::new::<degree>(90.0));

        let controller = SlavePullerSpeedController::new(
            Angle::new::<degree>(55.0), // target angle
            Angle::new::<degree>(35.0), // sensitivity (range around target)
            converter,
            filament_calc,
        );

        assert_eq!(controller.get_target_angle().get::<degree>(), 55.0);
        assert_eq!(controller.get_sensitivity().get::<degree>(), 35.0);
    }

    #[test]
    fn test_slave_puller_enabled_state() {
        let converter =
            LinearStepConverter::from_circumference(200, Length::new::<millimeter>(50.0));
        let filament_calc =
            FilamentTensionCalculator::new(Angle::new::<degree>(20.0), Angle::new::<degree>(90.0));

        let mut controller = SlavePullerSpeedController::new(
            Angle::new::<degree>(55.0),
            Angle::new::<degree>(35.0),
            converter,
            filament_calc,
        );

        assert!(!controller.is_enabled());

        controller.set_enabled(true);
        assert!(controller.is_enabled());

        controller.set_enabled(false);
        assert!(!controller.is_enabled());
    }

    #[test]
    fn test_optional_speed_factors() {
        let converter =
            LinearStepConverter::from_circumference(200, Length::new::<millimeter>(50.0));
        let filament_calc =
            FilamentTensionCalculator::new(Angle::new::<degree>(20.0), Angle::new::<degree>(90.0));

        let mut controller = SlavePullerSpeedController::new(
            Angle::new::<degree>(55.0),
            Angle::new::<degree>(35.0),
            converter,
            filament_calc,
        );

        assert_eq!(controller.get_min_speed_factor(), None);
        assert_eq!(controller.get_max_speed_factor(), None);

        controller.set_min_speed_factor(Some(0.7));
        controller.set_max_speed_factor(Some(1.5));

        assert_eq!(controller.get_min_speed_factor(), Some(0.7));
        assert_eq!(controller.get_max_speed_factor(), Some(1.5));
    }
}
