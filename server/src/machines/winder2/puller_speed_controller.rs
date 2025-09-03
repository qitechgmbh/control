use std::time::Instant;

use control_core::{
    controllers::{
        clamping_timeagnostic_pid::ClampingTimeagnosticPidController,
        second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    },
    converters::linear_step_converter::LinearStepConverter,
    uom_extensions::{
        acceleration::meter_per_minute_per_second, jerk::meter_per_minute_per_second_squared,
        velocity::meter_per_minute,
    },
};
use serde::{Deserialize, Serialize};
use uom::{
    ConstZero,
    si::{
        f64::{Acceleration, AngularVelocity, Jerk, Length, Velocity},
        length::millimeter,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum GearRatio {
    OneToOne,
    OneToFive,
    OneToTen,
}

impl GearRatio {
    /// Get the speed multiplier for this gear ratio
    pub fn multiplier(&self) -> f64 {
        match self {
            GearRatio::OneToOne => 1.0,
            GearRatio::OneToFive => 5.0,
            GearRatio::OneToTen => 10.0,
        }
    }
}

impl Default for GearRatio {
    fn default() -> Self {
        GearRatio::OneToOne
    }
}
use crate::machines::laser::api::PidSettings;

#[derive(Debug)]
pub struct PullerSpeedController {
    enabled: bool,
    pub target_speed: Velocity,
    pub target_diameter: Length,
    pub measured_diameter: Length,
    pub regulation_mode: PullerRegulationMode,
    /// Forward rotation direction. If false, applies negative sign to speed
    pub forward: bool,
    /// Gear ratio for winding speed (1:5 or 1:10)
    pub gear_ratio: GearRatio,
    /// Linear acceleration controller to dampen speed change
    acceleration_controller: LinearJerkSpeedController,
    /// Converter for linear to angular transformations
    pub converter: LinearStepConverter,
    pub last_speed: Velocity,
    pub pid: ClampingTimeagnosticPidController,
}

impl PullerSpeedController {
    pub fn new(
        target_speed: Velocity,
        target_diameter: Length,
        converter: LinearStepConverter,
    ) -> Self {
        let acceleration = Acceleration::new::<meter_per_minute_per_second>(5.0);
        let jerk = Jerk::new::<meter_per_minute_per_second_squared>(10.0);
        let speed = Velocity::new::<meter_per_minute>(50.0);

        Self {
            enabled: false,
            target_speed,
            target_diameter,
            measured_diameter: Length::ZERO,
            regulation_mode: PullerRegulationMode::Speed,
            forward: true,
            gear_ratio: GearRatio::default(),
            acceleration_controller: LinearJerkSpeedController::new_simple(
                Some(speed),
                acceleration,
                jerk,
            ),
            converter,
            last_speed: Velocity::ZERO,
            pid: ClampingTimeagnosticPidController::simple_new(0.01, 0.0, 0.02),
        }
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_target_speed(&mut self, target: Velocity) {
        self.target_speed = target;
    }

    pub fn set_target_diameter(&mut self, target: Length) {
        self.target_diameter = target;
    }

    pub const fn set_regulation_mode(&mut self, regulation: PullerRegulationMode) {
        self.regulation_mode = regulation;
    }

    pub const fn set_forward(&mut self, forward: bool) {
        self.forward = forward;
    }

    pub const fn set_gear_ratio(&mut self, gear_ratio: GearRatio) {
        self.gear_ratio = gear_ratio;
    }

    pub const fn get_gear_ratio(&self) -> GearRatio {
        self.gear_ratio
    }

    fn update_speed(&mut self, t: Instant) -> Velocity {
        let base_speed = match self.enabled {
            true => match self.regulation_mode {
                PullerRegulationMode::Speed => self.target_speed,
                PullerRegulationMode::Diameter => self.speed_from_diameter(t),
            },
            false => Velocity::ZERO,
        };

        // Apply gear ratio multiplier
        let speed = base_speed * self.gear_ratio.multiplier();

        let speed = if self.forward { speed } else { -speed };

        let speed = self.acceleration_controller.update(speed, t);

        self.last_speed = speed;
        speed
    }

    fn speed_from_diameter(&mut self, now: Instant) -> Velocity {
        let error = self.target_diameter - self.measured_diameter;
        let speed_change = self.pid.update(error.get::<millimeter>(), now);

        self.target_speed += Velocity::new::<meter_per_minute>(speed_change);
        Self::clamp_speed(
            self.target_speed,
            Velocity::new::<meter_per_minute>(0.0),
            Velocity::new::<meter_per_minute>(100.0),
        )
    }

    fn clamp_speed(speed: Velocity, min: Velocity, max: Velocity) -> Velocity {
        if speed < min {
            min
        } else if speed > max {
            max
        } else {
            speed
        }
    }

    pub fn speed_to_angular_velocity(&self, speed: Velocity) -> AngularVelocity {
        // Use the converter to transform from linear velocity to angular velocity
        self.converter.velocity_to_angular_velocity(speed)
    }

    pub fn angular_velocity_to_speed(&self, angular_speed: AngularVelocity) -> Velocity {
        // Use the converter to transform from angular velocity to linear velocity
        self.converter.angular_velocity_to_velocity(angular_speed)
    }

    pub fn calc_angular_velocity(&mut self, t: Instant) -> AngularVelocity {
        let speed = self.update_speed(t);
        self.speed_to_angular_velocity(speed)
    }

    pub fn get_target_speed(&self) -> Velocity {
        self.target_speed
    }

    pub fn get_pid_params(&self) -> PidSettings {
        PidSettings {
            ki: self.pid.get_ki(),
            kp: self.pid.get_kp(),
            kd: self.pid.get_kd(),
            dead: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum PullerRegulationMode {
    #[default]
    Speed,
    Diameter,
}
