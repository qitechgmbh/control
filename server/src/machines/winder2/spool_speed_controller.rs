use std::{fmt::Debug, time::Instant};

use super::tension_arm::TensionArm;

pub trait SpoolSpeedControllerTrait
where
    Self: Debug,
{
    fn get_speed(&mut self, t: Instant, tension_arm: &TensionArm) -> i32;
    fn reset(&mut self);
    fn set_max_speed(&mut self, max_speed: f64);
    fn set_min_speed(&mut self, min_speed: f64);
    fn get_max_speed(&self) -> f64;
    fn get_min_speed(&self) -> f64;
    fn set_enabled(&mut self, enabled: bool);
}
