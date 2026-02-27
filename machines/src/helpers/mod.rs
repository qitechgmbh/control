// keep for backwards reference
mod clamp_revolution_old;

mod clamp_revolution;
pub use clamp_revolution::clamp_revolution;
pub use clamp_revolution::clamp_revolution_uom;
pub use clamp_revolution::scale_revolution_to_range;

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