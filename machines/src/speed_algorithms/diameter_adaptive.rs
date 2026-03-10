use std::time::Instant;

use units::{ConstZero, Velocity, Length};
use units::velocity::meter_per_second;
use units::length::{meter, millimeter};

use crate::speed_algorithms::BoundedValue;

/// Computes output speed by waiting until the adjustment distance is reached,
/// then nudging the modulation step-wise toward the target.
#[derive(Debug, Clone)]
pub struct AdaptiveDiameterSpeedAlgorithm {
    // config
    speed_base:          BoundedValue<Velocity>,
    speed_delta_max:     BoundedValue<f64>,
    increase_per_step:   BoundedValue<f64>,
    tolerance_limit:     BoundedValue<Length>,
    adjustment_distance: BoundedValue<Length>,

    // internal state
    modulation: f64,
    distance_since_last_adjustment: Length,
    time_since_last_update: Instant,
}

// public interface
impl AdaptiveDiameterSpeedAlgorithm 
{
    pub fn new(
        speed_base:          BoundedValue<Velocity>,
        speed_delta_max:     BoundedValue<f64>,
        increase_per_step:   BoundedValue<f64>,
        tolerance_limit:     BoundedValue<Length>,
        adjustment_distance: BoundedValue<Length>,
    ) -> Self
    {
        Self { 
            // config
            speed_base,
            speed_delta_max,
            increase_per_step,
            tolerance_limit, 
            adjustment_distance, 
            // state
            modulation: 0.0, 
            distance_since_last_adjustment: Length::ZERO, 
            time_since_last_update: Instant::now(),
        }
    }

    pub fn desired_speed(&self) -> Velocity
    {
        let factor = 1.0 + self.modulation * self.speed_delta_max.get();
        (self.speed_base.get() * factor).max(Velocity::ZERO)
    }

    pub fn update(&mut self, dt: f64, speed_prev: Velocity, data: Option<DiameterData>)
    {
        let Some(data) = data else {
            self.reset();
            return; 
        };

        let error = data.current - data.target;
        if error.abs() <= self.tolerance_limit.get()
        {
            // error within tolerance
            self.distance_since_last_adjustment = Length::ZERO;
            return;
        }

        if !self.accumulate_distance(dt, speed_prev) {
            // adjustment distance not reached yet
            return;
        }

        // apply modulation step in the direction of the error
        let correction_sign = error.get::<millimeter>().signum();
        let modulation_step = self.increase_per_step.get() * correction_sign;
        self.modulation = (self.modulation + modulation_step).clamp(-1.0, 1.0);
    }
}

// getters + setters
impl AdaptiveDiameterSpeedAlgorithm {
    pub fn speed_delta_max(&self) -> f64 {
        self.speed_delta_max.get()
    }

    pub fn set_speed_delta_max(&mut self, value: f64) {
        // up to 50%
        self.speed_delta_max.set(value);
    }

    pub fn increase_per_step(&self) -> f64 {
        self.increase_per_step.get()
    }

    pub fn set_increase_per_step(&mut self, value: f64) {
        // up to 10%
        self.increase_per_step.set(value);
    }

    pub fn adjustment_distance(&self) -> Length {
        self.adjustment_distance.get()
    }

    pub fn set_adjustment_distance(&mut self, value: Length) {
        self.adjustment_distance.set(value);
    }

    pub fn tolerance_limit(&self) -> Length {
        self.tolerance_limit.get()
    }

    pub fn set_tolerance_limit(&mut self, value: Length) {
        self.tolerance_limit.set(value);
    }

    /// Current modulation level in [-1.0, 1.0].
    pub fn modulation(&self) -> f64 {
        self.modulation
    }

    /// Reset modulation and accumulated distance to zero.
    pub fn reset(&mut self) {
        self.modulation = 0.0;
        self.distance_since_last_adjustment = Length::ZERO;
    }
}

// helpers
impl AdaptiveDiameterSpeedAlgorithm {
    /// Accumulate distance; returns `true` if adjustment is due.
    fn accumulate_distance(&mut self, dt: f64, speed_prev: Velocity) -> bool {
        let meters_added = speed_prev.abs().get::<meter_per_second>() * dt;
        self.distance_since_last_adjustment += Length::new::<meter>(meters_added);

        if self.distance_since_last_adjustment >= self.adjustment_distance.get() {
            self.distance_since_last_adjustment = Length::ZERO;
            true
        } else {
            false
        }
    }
}

pub struct DiameterData
{
    current: Length,
    target:  Length,
    lower:   Length,
    upper:   Length,
}