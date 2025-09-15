use std::{
    f64::consts::PI,
    time::{Duration, Instant},
};

use control_core::{
    controllers::{
        deadtime_p_controller::TimeAgnosticDeadTimePController,
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
        length::{meter, millimeter},
        time::second,
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
use crate::machines::winder2::api::PidSettings;

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
    pub p_dead_controller: TimeAgnosticDeadTimePController,
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
            p_dead_controller: TimeAgnosticDeadTimePController::new(0.1 / 10000.0, Duration::ZERO),
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

    pub fn set_measured_diameter(&mut self, target: f64) {
        self.measured_diameter = Length::new::<millimeter>(target);
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
<<<<<<< HEAD
        let base_speed = match self.enabled {
            true => match self.regulation_mode {
                PullerRegulationMode::Speed => self.target_speed,
                PullerRegulationMode::Diameter => {
                    self.speed_from_diameter(t);
=======
        if !self.enabled {
            return Velocity::ZERO;
        }

        let target_speed = match self.regulation_mode {
            PullerRegulationMode::Speed => {
                let directional_target = if self.forward {
>>>>>>> 78962097 (Implemented deadtime_p_controller to regulate puller speed by filament)
                    self.target_speed
                } else {
                    -self.target_speed
                };
                self.acceleration_controller.update(directional_target, t)
            }
            PullerRegulationMode::Diameter => {
                let diameter_speed = self.speed_from_diameter(t);
                if self.forward {
                    diameter_speed
                } else {
                    -diameter_speed
                }
            }
            PullerRegulationMode::DiameterNoPid => {
                let diameter_speed = self.calculate_target_speed();
                if self.forward {
                    diameter_speed
                } else {
                    -diameter_speed
                }
            }
        };

<<<<<<< HEAD
        // Apply gear ratio multiplier
        let speed = base_speed * self.gear_ratio.multiplier();

        let speed = if self.forward { speed } else { -speed };

        let speed = self.acceleration_controller.update(speed, t);

        self.last_speed = speed;
        speed
=======
        self.last_speed = target_speed;
        target_speed
>>>>>>> 78962097 (Implemented deadtime_p_controller to regulate puller speed by filament)
    }

    fn calculate_target_speed(&self) -> Velocity {
        //
        // Q_measured = (PI*d^2 / 4) * v
        // v_new = (4 * Q) / PI * d^2
        // 
        if self.measured_diameter < Length::new::<millimeter>(0.1) {
            return Velocity::new::<meter_per_minute>(0.5);
        }
        let q_meas = (PI * self.measured_diameter * self.measured_diameter / 4.0) * self.last_speed;

        (4.0 * q_meas) / (PI * self.target_diameter * self.target_diameter)
    }

    fn speed_from_diameter(&mut self, now: Instant) -> Velocity {
        // get current error
        let error =
            self.measured_diameter.get::<millimeter>() - self.target_diameter.get::<millimeter>();

        // calculate/set deadtime based of speed and distance
        let deadtime = Self::calc_deadtime(self.last_speed, Length::new::<meter>(2.0));
        self.p_dead_controller.set_dead(deadtime);

        // get speed change from p controller
        let speed_change = self.p_dead_controller.update(error, now);

        // apply speed change to target speed
        let next_speed = self.last_speed + Velocity::new::<meter_per_minute>(speed_change);

        // clamp the speed to 0 - 50 for safety
        Self::clamp_speed(
            next_speed,
            Velocity::new::<meter_per_minute>(0.0),
            Velocity::new::<meter_per_minute>(50.0),
        )
    }

    fn calc_deadtime(speed: Velocity, distance: Length) -> Duration {
        // check if speed is 0 => This should not happen
        if speed <= Velocity::ZERO {
            return Duration::from_secs(180);
        }
        // get the duration based of distance and speed
        let secs = (distance / speed).get::<second>().max(0.0);
        Duration::from_secs_f64(secs)
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
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum PullerRegulationMode {
    #[default]
    Speed,
    Diameter,
    DiameterNoPid,
}
