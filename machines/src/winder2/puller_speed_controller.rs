use std::time::Instant;

use control_core::{
    controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    converters::linear_step_converter::LinearStepConverter,
};
use serde::{Deserialize, Serialize};
use units::ConstZero;
use units::acceleration::meter_per_minute_per_second;
use units::f64::*;
use units::jerk::meter_per_minute_per_second_squared;
use units::velocity::meter_per_minute;

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

#[derive(Debug)]
pub struct PullerSpeedController {
    enabled: bool,
    pub target_speed: Velocity,

    pub adaptive: AdaptiveSpeedAlgorithm,

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
}

impl PullerSpeedController {
    pub fn new(
        target_speed: Velocity,
        converter: LinearStepConverter,
    ) -> Self {
        let acceleration = Acceleration::new::<meter_per_minute_per_second>(5.0);
        let jerk = Jerk::new::<meter_per_minute_per_second_squared>(10.0);
        let speed = Velocity::new::<meter_per_minute>(50.0);

        let adaptive = AdaptiveSpeedAlgorithm {
            speed_max:       speed,
            speed_base:      target_speed,
            deviation_limit: Velocity::ZERO,
            modulation:      0.0,
        };

        Self {
            enabled: false,
            target_speed,
            adaptive,
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
        }
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_target_speed(&mut self, target: Velocity) {
        self.target_speed = target;
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
                PullerRegulationMode::Diameter => self.adaptive.compute(),
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
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum PullerRegulationMode {
    #[default]
    Speed,
    Diameter,
}

#[derive(Debug, Clone, Copy)]
pub struct AdaptiveSpeedAlgorithm 
{
    speed_max:  Velocity,
    speed_base: Velocity,
    deviation_limit: Velocity,
    modulation: f64, // (-1.0 to 1.0)
}

impl AdaptiveSpeedAlgorithm
{
    pub fn compute(&self) -> Velocity
    {
        self.speed_base + (self.deviation_limit * self.modulation)
    }

    pub fn speed_base(&self) -> Velocity {
        self.speed_base
    }

    pub fn set_speed_base(&mut self, speed: Velocity) {
        self.speed_base = speed
            .min(self.speed_max) // ensure < max
            .max(Velocity::ZERO) // ensure > 0
            ;

        // update deviation limit as well
        self.set_deviation_limit(self.deviation_limit);
    }

    pub fn deviation_limit(&self) -> Velocity {
        self.deviation_limit
    }

    pub fn set_deviation_limit(&mut self, deviation_limit: Velocity) 
    {
        self.deviation_limit = 
            // ensure > 0
            if deviation_limit < Velocity::ZERO
                { Velocity::ZERO }
            // ensure base - deviation can't below zero
            else if deviation_limit > self.speed_base 
                { self.speed_base } 
            // ensure base + deviation can't exceed max
            else if self.speed_base + deviation_limit > self.speed_max 
                { self.speed_max - self.speed_base }
            // in valid range
            else 
                { deviation_limit };
    }

    pub fn set_modulation(&mut self, modulation: f64) 
    {
        self.modulation = modulation.clamp(-1.0, 1.0);
    }
}