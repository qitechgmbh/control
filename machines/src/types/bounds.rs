use std::ops::{Div, Sub};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Bounds<T: Clone + Copy + PartialOrd> 
{
    pub min: T,
    pub max: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ExceededBound
{
    Min,
    Max,
}

impl<T: Copy + PartialOrd> Bounds<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    pub fn check(&self, value: T) -> Option<ExceededBound> {
        if value < self.min {
            Some(ExceededBound::Min)
        } else if value > self.max {
            Some(ExceededBound::Max)
        } else {
            None
        }
    }

    pub fn clamp(&self, value: T) -> T {
        if value < self.min {
            self.min
        } else if value > self.max {
            self.max
        } else {
            value
        }
    }
}

impl<T> Bounds<T>
where
    T: Copy + PartialOrd + Sub<Output = T> + Div<Output = T> ,
{
    /// Normalize a value within the bounds to [0.0, 1.0]
    pub fn normalize(&self, value: T) -> f64
    where
        T: Into<f64>,
    {
        
        let span   = self.max - self.min;
        let offset = value - self.min;

        let ratio = (offset / span).into();
        ratio.clamp(0.0, 1.0)
    }
}

impl<T: Clone + Copy + PartialOrd + Default> Default for Bounds<T> {
    fn default() -> Self {
        Self { min: T::default(), max: T::default() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct BoundedValue<T: Clone + Copy + PartialOrd>
{
    value:  T,
    bounds: Bounds<T>,
}

impl<T: Clone + Copy + PartialOrd> BoundedValue<T> 
{
    pub fn new(value: T, bounds: Bounds<T>) -> Self {
        Self { value, bounds }
    }

    pub fn bounds(self) -> Bounds<T> {
        self.bounds
    }

    pub fn set_bounds(self, value: T) -> Option<ExceededBound> {
        let result = self.bounds.check(value);
        self.bounds.clamp(value);
        result
    }

    pub fn value(self) -> T {
        self.value
    }

    pub fn set_value(self, value: T) -> Option<ExceededBound> {
        let result = self.bounds.check(value);
        self.bounds.clamp(value);
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct ClampResult<T: Clone + Copy + PartialOrd>
{
    value: T,
    exceeded_bound: Option<ExceededBound>
}

impl<T: Clone + Copy + PartialOrd> ClampResult<T> 
{
    pub fn new(value: T, exceeded_bound: Option<ExceededBound>) -> Self {
        Self { value, exceeded_bound }
    }
}