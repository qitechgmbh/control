use std::{
    collections::VecDeque,
    iter::Sum,
    time::{Duration, Instant},
};

/// A helper trait to handle division for averaging
pub trait DivideByCount {
    fn divide_by_count(self, count: usize) -> Self;
}

impl DivideByCount for f64 {
    fn divide_by_count(self, count: usize) -> Self {
        self / (count as Self)
    }
}

impl DivideByCount for f32 {
    fn divide_by_count(self, count: usize) -> Self {
        self / (count as Self)
    }
}

impl DivideByCount for i32 {
    fn divide_by_count(self, count: usize) -> Self {
        self / (count as Self)
    }
}

impl DivideByCount for i64 {
    fn divide_by_count(self, count: usize) -> Self {
        self / (count as Self)
    }
}

impl DivideByCount for u32 {
    fn divide_by_count(self, count: usize) -> Self {
        self / (count as Self)
    }
}

impl DivideByCount for u64 {
    fn divide_by_count(self, count: usize) -> Self {
        self / (count as Self)
    }
}

// Type aliases for common use cases
pub type F64Window = MovingTimeWindow<f64>;
pub type F32Window = MovingTimeWindow<f32>;
pub type I32Window = MovingTimeWindow<i32>;
pub type I64Window = MovingTimeWindow<i64>;

#[derive(Debug)]
pub struct MovingTimeWindow<T>
where
    T: Copy + PartialOrd + Sum + DivideByCount + std::fmt::Debug + Default,
{
    max_samples: usize,
    samples: VecDeque<(T, Instant)>,
    last_sample: Option<(T, Instant)>,
    sample_time: Duration,
    cached_average: Option<T>,
    cached_max: Option<T>,
    cached_min: Option<T>,
}

/// A time-based moving window that maintains samples within a specified duration and count limit.
///
/// This structure efficiently tracks values over time, automatically removing samples that fall
/// outside the configured time window or exceed the maximum sample count. It provides cached
/// average calculation to avoid redundant computations.
impl<T> MovingTimeWindow<T>
where
    T: Copy + PartialOrd + Sum + DivideByCount + std::fmt::Debug + Default,
{
    /// Creates a new moving time window with the specified duration and maximum sample count.
    ///
    /// # Arguments
    ///
    /// * `duration` - The time window duration; samples older than this will be automatically removed
    /// * `samples` - Maximum number of samples to retain, regardless of their age
    ///
    /// # Returns
    ///
    /// A new `MovingTimeWindow` instance with empty sample collection and initialized cache
    pub fn new(duration: Duration, max_samples: usize) -> Self {
        Self {
            max_samples,
            samples: VecDeque::with_capacity(max_samples),
            last_sample: None,
            sample_time: duration / (max_samples as u32),
            cached_average: None,
            cached_max: None,
            cached_min: None,
        }
    }

    /// Updates the window with a new sample value at the given timestamp.
    ///
    /// This method adds the new sample to the window and automatically removes samples that:
    /// - Are older than the configured duration from the current timestamp
    /// - Exceed the maximum sample count limit
    ///
    /// The cached average is invalidated after this operation.
    ///
    /// # Arguments
    ///
    /// * `value` - The numeric value to add to the window
    /// * `timestamp` - The time at which this sample was taken
    pub fn update(&mut self, value: T, timestamp: Instant) {
        // Chekc if we should add a new sample based on the sample time and the last sample
        if let Some((_, last_timestamp)) = self.last_sample {
            if timestamp.duration_since(last_timestamp) < self.sample_time {
                // If the new sample is too close to the last one, we skip adding it
                return;
            }
        }

        // If we have reached the maximum sample count, remove the oldest sample
        if self.samples.len() >= self.max_samples {
            self.samples.pop_back();
        }

        // Add the new sample
        self.samples.push_front((value, timestamp));
        self.last_sample = Some((value, timestamp));

        // Invalidate all cached values since we added a new sample
        self.cached_average = None;
        self.cached_max = None;
        self.cached_min = None;
    }

    /// Calculates and returns the average of all samples currently in the window.
    ///
    /// This method uses caching to avoid redundant calculations. The average is only
    /// recalculated when the window contents have changed since the last call.
    ///
    /// # Returns
    ///
    /// The arithmetic mean of all current samples, or T::default() if the window is empty
    pub fn average(&mut self) -> T {
        // Return cached value if available
        if let Some(cached) = self.cached_average {
            return cached;
        }

        let sum: T = self.samples.iter().map(|(v, _)| *v).sum();
        let count = self.samples.len();

        let avg = if count > 0 {
            sum.divide_by_count(count)
        } else {
            T::default()
        };

        // Cache the calculated value
        self.cached_average = Some(avg);
        avg
    }

    /// Calculates and returns the maximum value of all samples currently in the window.
    ///
    /// This method uses caching to avoid redundant calculations. The maximum is only
    /// recalculated when the window contents have changed since the last call.
    ///
    /// # Returns
    ///
    /// The maximum value of all current samples, or T::default() if the window is empty
    pub fn max(&mut self) -> T {
        // Return cached value if available
        if let Some(cached) = self.cached_max {
            return cached;
        }

        let result = self
            .samples
            .iter()
            .map(|(v, _)| *v)
            .fold(None, |acc, val| {
                acc.map_or(Some(val), |current_max| {
                    if val > current_max {
                        Some(val)
                    } else {
                        Some(current_max)
                    }
                })
            })
            .unwrap_or_default();

        // Cache the calculated value
        self.cached_max = Some(result);
        result
    }

    /// Calculates and returns the minimum value of all samples currently in the window.
    ///
    /// This method uses caching to avoid redundant calculations. The minimum is only
    /// recalculated when the window contents have changed since the last call.
    ///
    /// # Returns
    ///
    /// The minimum value of all current samples, or T::default() if the window is empty
    pub fn min(&mut self) -> T {
        // Return cached value if available
        if let Some(cached) = self.cached_min {
            return cached;
        }

        let result = self
            .samples
            .iter()
            .map(|(v, _)| *v)
            .fold(None, |acc, val| {
                acc.map_or(Some(val), |current_min| {
                    if val < current_min {
                        Some(val)
                    } else {
                        Some(current_min)
                    }
                })
            })
            .unwrap_or_default();

        // Cache the calculated value
        self.cached_min = Some(result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic() {
        // Test that the struct compiles with basic numeric types
        let _: MovingTimeWindow<f64> = MovingTimeWindow::new(Duration::from_secs(1), 1);
        let _: MovingTimeWindow<f32> = MovingTimeWindow::new(Duration::from_secs(1), 1);
        let _: MovingTimeWindow<i32> = MovingTimeWindow::new(Duration::from_secs(1), 1);
    }
}
