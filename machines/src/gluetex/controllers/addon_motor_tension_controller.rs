use std::time::Instant;

use units::ConstZero;
use units::angle::degree;
use units::f64::*;
use units::velocity::meter_per_second;

use super::tension_arm::TensionArm;

/// Tension-based speed controller for addon motor 5.
///
/// Adjusts speed around a target angle using the same behavior as the slave puller,
/// while preserving the master speed direction.
#[derive(Debug)]
pub struct AddonMotorTensionController {
    /// Last commanded velocity
    last_speed: Velocity,
    /// Whether tension control is enabled
    enabled: bool,
    /// Target angle for the tension arm (setpoint)
    target_angle: Angle,
    /// Sensitivity range around target angle (in degrees)
    sensitivity: Angle,
    /// Optional minimum speed factor for overspeed protection (e.g., 0.5 = 50%)
    min_speed_factor: Option<f64>,
    /// Optional maximum speed factor for overspeed protection (e.g., 2.0 = 200%)
    max_speed_factor: Option<f64>,
}

impl AddonMotorTensionController {
    /// Create a new addon motor tension controller
    pub fn new(target_angle: Angle, sensitivity: Angle) -> Self {
        Self {
            last_speed: Velocity::ZERO,
            enabled: false,
            target_angle,
            sensitivity,
            min_speed_factor: None,
            max_speed_factor: None,
        }
    }

    /// Calculate the raw target speed based on master speed and tension arm angle
    fn speed_raw(&self, master_speed: Velocity, tension_arm: &TensionArm) -> Velocity {
        let master_abs = master_speed.abs();
        let sensitivity_deg = self.sensitivity.get::<degree>();

        if sensitivity_deg == 0.0 {
            return master_speed;
        }

        // Get tension arm angle
        let current_angle = tension_arm.get_angle();

        // Calculate deviation from target angle
        let deviation = current_angle - self.target_angle;

        // Normalize deviation to [-1.0, 1.0] based on sensitivity range
        let normalized_deviation = (deviation.get::<degree>() / sensitivity_deg).clamp(-1.0, 1.0);

        // Calculate speed factor:
        // -1.0 (below target) -> 0.5 (50% speed - slow down)
        //  0.0 (at target)    -> 1.0 (100% speed)
        // +1.0 (above target) -> 1.5 (150% speed - speed up)
        let speed_factor = 1.0 + (normalized_deviation * 0.5);

        let base_speed = master_abs * speed_factor;

        // Apply optional speed factor limits for overspeed protection
        let limited_speed = if let Some(min_factor) = self.min_speed_factor {
            base_speed.max(master_abs * min_factor)
        } else {
            base_speed
        };

        let limited_speed = if let Some(max_factor) = self.max_speed_factor {
            limited_speed.min(master_abs * max_factor)
        } else {
            limited_speed
        };

        // Preserve master speed direction
        if master_speed.get::<meter_per_second>() < 0.0 {
            -limited_speed
        } else {
            limited_speed
        }
    }

    /// Main update function called every control cycle
    pub fn update_speed(
        &mut self,
        _t: Instant,
        master_speed: Velocity,
        tension_arm: &TensionArm,
    ) -> Velocity {
        let final_speed = if self.enabled {
            self.speed_raw(master_speed, tension_arm)
        } else {
            master_speed
        };

        self.last_speed = final_speed;
        final_speed
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethercat_hal::io::{
        analog_input::{AnalogInputInput, physical::AnalogInputRange},
        analog_input_dummy::AnalogInputDummy,
    };
    use std::i16;
    use std::time::Instant;
    use units::angle::degree;
    use units::electric_potential::volt;
    use units::f64::ElectricPotential;

    #[test]
    fn test_addon_tension_controller_defaults() {
        let controller = AddonMotorTensionController::new(
            Angle::new::<degree>(55.0),
            Angle::new::<degree>(35.0),
        );

        assert_eq!(controller.get_target_angle().get::<degree>(), 55.0);
        assert_eq!(controller.get_sensitivity().get::<degree>(), 35.0);
        assert!(!controller.is_enabled());
        assert!(controller.get_min_speed_factor().is_none());
        assert!(controller.get_max_speed_factor().is_none());
    }

    #[test]
    fn test_addon_tension_controller_disabled_passthrough() {
        let mut controller = AddonMotorTensionController::new(
            Angle::new::<degree>(55.0),
            Angle::new::<degree>(35.0),
        );

        let mut analog_input_dummy = AnalogInputDummy::new(AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(10.0),
            min_raw: 0,
            max_raw: i16::MAX,
        });
        analog_input_dummy.set_input(AnalogInputInput {
            normalized: 0.0,
            wiring_error: false,
        });
        let tension_arm = TensionArm::new(analog_input_dummy.analog_input());

        let speed = Velocity::new::<meter_per_second>(1.0);
        let output = controller.update_speed(Instant::now(), speed, &tension_arm);

        assert_eq!(output.get::<meter_per_second>(), 1.0);
    }

    #[test]
    fn test_addon_tension_controller_limits() {
        let mut controller = AddonMotorTensionController::new(
            Angle::new::<degree>(55.0),
            Angle::new::<degree>(35.0),
        );
        controller.set_enabled(true);
        controller.set_min_speed_factor(Some(0.5));
        controller.set_max_speed_factor(Some(1.5));

        let mut analog_input_dummy = AnalogInputDummy::new(AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(10.0),
            min_raw: 0,
            max_raw: i16::MAX,
        });
        // 3.75V = 0.75 revolution -> 270deg
        analog_input_dummy.set_input(AnalogInputInput {
            normalized: 3.75 / 10.0,
            wiring_error: false,
        });
        let tension_arm = TensionArm::new(analog_input_dummy.analog_input());

        let speed = Velocity::new::<meter_per_second>(2.0);
        let output = controller.update_speed(Instant::now(), speed, &tension_arm);

        // Max speed factor should clamp to 1.5x
        assert_eq!(output.get::<meter_per_second>(), 3.0);
    }
}
