#[derive(Debug, Clone, Copy)]
pub struct BoundedValue<T> {
    min: T,
    max: T,
    current: T,
}

impl<T> BoundedValue<T>
where
    T: PartialOrd + Copy,
{
    /// Create a new BoundedValue; clamps initial to [min, max]
    pub fn new(min: T, max: T, initial: T) -> Self {
        let current = Self::clamp(initial, min, max);
        Self { min, max, current }
    }

    /// Get current value
    pub fn get(&self) -> T {
        self.current
    }

    /// Set current value, clamped
    pub fn set(&mut self, value: T) {
        self.current = Self::clamp(value, self.min, self.max);
    }

    /// Increment current value, clamped
    pub fn add(&mut self, delta: T)
    where
        T: std::ops::Add<Output = T>,
    {
        self.current = Self::clamp(self.current + delta, self.min, self.max);
    }

    /// Decrement current value, clamped
    pub fn sub(&mut self, delta: T)
    where
        T: std::ops::Sub<Output = T>,
    {
        self.current = Self::clamp(self.current - delta, self.min, self.max);
    }

    /// Set a new minimum, adjusting current if needed
    pub fn set_min(&mut self, min: T) {
        self.min = min;
        if self.current < min {
            self.current = min;
        }
    }

    /// Set a new maximum, adjusting current if needed
    pub fn set_max(&mut self, max: T) {
        self.max = max;
        if self.current > max {
            self.current = max;
        }
    }

    /// Get (min, max) tuple
    pub fn range(&self) -> (T, T) {
        (self.min, self.max)
    }

    /// Helper clamp function for generic T
    fn clamp(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
}