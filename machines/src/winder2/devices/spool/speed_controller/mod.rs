use std::time::Instant;

use units::AngularVelocity;

mod helpers;

mod min_max;
pub use min_max::MinMaxSpeedController;

mod adapative;
pub use adapative::AdaptiveSpeedController;

use crate::winder2::devices::{Puller, TensionArm};

pub trait SpeedController
{
    fn speed(&self) -> AngularVelocity;
    fn set_speed(&mut self, speed: AngularVelocity);

    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);

    fn reset(&mut self);

    fn update_speed(
        &mut self, 
        t: Instant,
        multiplier: f64,
        tension_arm: &TensionArm, 
        puller: &Puller
    ) -> AngularVelocity;
}