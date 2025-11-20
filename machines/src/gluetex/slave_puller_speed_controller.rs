use std::time::Instant;

use control_core::converters::linear_step_converter::LinearStepConverter;
use units::ConstZero;
use units::f64::*;
use units::velocity::meter_per_minute;

use super::{
    clamp_revolution::Clamping, clamp_revolution::clamp_revolution_uom,
    filament_tension::FilamentTensionCalculator, tension_arm::TensionArm,
};

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
    /// Minimum angle for detection zone (low tension, high speed)
    min_angle: Angle,
    /// Maximum angle for detection zone (high tension, low speed)
    max_angle: Angle,
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
    /// * `min_angle` - Minimum angle for detection zone (low tension, high speed)
    /// * `max_angle` - Maximum angle for detection zone (high tension, low speed)
    /// * `converter` - Linear step converter for the motor
    /// * `filament_calc` - Tension calculator for the tension arm
    pub fn new(
        min_angle: Angle,
        max_angle: Angle,
        converter: LinearStepConverter,
        filament_calc: FilamentTensionCalculator,
    ) -> Self {
        Self {
            last_speed: Velocity::ZERO,
            enabled: false,
            forward: true,
            min_angle,
            max_angle,
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
        // Get tension arm angle and clamp to valid range
        let tension_arm_angle = tension_arm.get_angle();
        let (clamped_angle, clamping_state) = clamp_revolution_uom(
            tension_arm_angle,
            self.max_angle, // Inverted: min angle = max tension = low speed
            self.min_angle, // Inverted: max angle = min tension = high speed
        );

        // If at limits, return master speed (no adjustment)
        let base_speed = match clamping_state {
            Clamping::Min => master_speed, // At max angle (high tension) - return base speed
            Clamping::Max => master_speed, // At min angle (low tension) - return base speed
            _ => {
                // Calculate normalized tension (0.0 = low tension, 1.0 = high tension)
                let filament_tension = self.filament_calc.calc_filament_tension(clamped_angle);

                // Invert tension for speed calculation: low tension = high speed, high tension = low speed
                // At min_angle (low tension): tension = 0.0, inverted = 1.0 -> slave runs faster
                // At max_angle (high tension): tension = 1.0, inverted = 0.0 -> slave runs slower
                let speed_factor = 1.0 - filament_tension;

                // Interpolate between 50% and 150% of master speed based on tension
                // This gives a natural range that can be further limited by optional factors
                let factor = 0.5 + speed_factor; // Maps 0.0-1.0 to 0.5-1.5

                master_speed * factor
            }
        };

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
        t: Instant,
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

    pub const fn set_min_angle(&mut self, angle: Angle) {
        self.min_angle = angle;
    }

    pub const fn get_min_angle(&self) -> Angle {
        self.min_angle
    }

    pub const fn set_max_angle(&mut self, angle: Angle) {
        self.max_angle = angle;
    }

    pub const fn get_max_angle(&self) -> Angle {
        self.max_angle
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
        let converter = LinearStepConverter::new(200, Length::new::<millimeter>(50.0));
        let filament_calc =
            FilamentTensionCalculator::new(Angle::new::<degree>(20.0), Angle::new::<degree>(90.0));

        let controller = SlavePullerSpeedController::new(
            Angle::new::<degree>(20.0),
            Angle::new::<degree>(90.0),
            converter,
            filament_calc,
        );

        assert_eq!(controller.get_min_angle().get::<degree>(), 20.0);
        assert_eq!(controller.get_max_angle().get::<degree>(), 90.0);
    }

    #[test]
    fn test_slave_puller_enabled_state() {
        let converter = LinearStepConverter::new(200, Length::new::<millimeter>(50.0));
        let filament_calc =
            FilamentTensionCalculator::new(Angle::new::<degree>(20.0), Angle::new::<degree>(90.0));

        let mut controller = SlavePullerSpeedController::new(
            Angle::new::<degree>(20.0),
            Angle::new::<degree>(90.0),
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
        let converter = LinearStepConverter::new(200, Length::new::<millimeter>(50.0));
        let filament_calc =
            FilamentTensionCalculator::new(Angle::new::<degree>(20.0), Angle::new::<degree>(90.0));

        let mut controller = SlavePullerSpeedController::new(
            Angle::new::<degree>(20.0),
            Angle::new::<degree>(90.0),
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
