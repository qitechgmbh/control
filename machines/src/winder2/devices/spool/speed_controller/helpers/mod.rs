mod filament_tension_calculator;
pub use filament_tension_calculator::FilamentTensionCalculator;

mod clamp_revolution;
pub use clamp_revolution::clamp_revolution_uom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clamp
{
    None,
    Min,
    Max,
}

pub struct ClampedValue<T>
{
    pub value: T,
    pub clamp: Clamp,
}

impl<T> ClampedValue<T>
{
    pub fn new(value: T, clamp: Clamp) -> Self
    {
        Self { value, clamp }
    }
}