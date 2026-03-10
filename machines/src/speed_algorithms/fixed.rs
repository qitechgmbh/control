use units::f64::Velocity;

use crate::speed_algorithms::BoundedValue;

#[derive(Debug)]
pub struct FixedSpeedAlgorithm
{
    speed_target: BoundedValue<Velocity>,
}

impl FixedSpeedAlgorithm
{
    pub fn new(speed_target: BoundedValue<Velocity>) -> Self {
        Self { speed_target}
    }

    pub fn compute(&self) -> Velocity {
        self.speed_target()
    }

    pub fn speed_target(&self) -> Velocity {
        self.speed_target.get()
    }

    pub fn set_speed_target(&mut self, value: Velocity) {
        self.speed_target.set(value);
    }
}