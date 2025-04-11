use std::fmt::Debug;

use super::tension_arm::TensionArm;

pub trait SpoolSpeedControllerTrait
where
    Self: Debug,
{
    fn get_speed(&mut self, nanoseconds: u64, tension_arm: &TensionArm) -> i32;
    fn reset(&mut self);
    fn set_max_speed(&mut self, max_speed: f32);
    fn set_min_speed(&mut self, min_speed: f32);
    fn get_max_speed(&self) -> f32;
    fn get_min_speed(&self) -> f32;
    fn set_enabled(&mut self, enabled: bool);
}
