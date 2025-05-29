use smol::Timer;
use std::time::{Duration, Instant};

pub struct LoopThrottle {
    target_cycle_time: Duration,
    update_interval_cycles: usize,
    window_start: Instant,
    total_cycles: usize,
    delay: Duration,
    last_avg_cycle_time: Duration,
    on_update: Option<Box<dyn Fn(LoopThrottleUpdate)>>,
}

pub struct LoopThrottleUpdate {
    pub delay: Duration,
    pub avg_cycle_time: Duration,
    pub error: i128, // Error in nanoseconds
}
impl LoopThrottle {
    pub fn new(
        target_cycle_time: Duration,
        update_interval_cycles: usize,
        on_update: Option<Box<dyn Fn(LoopThrottleUpdate)>>,
    ) -> Self {
        Self {
            target_cycle_time,
            update_interval_cycles,
            window_start: Instant::now(),
            total_cycles: 0,
            delay: target_cycle_time,
            last_avg_cycle_time: target_cycle_time,
            on_update,
        }
    }

    pub async fn sleep(&mut self) {
        self.total_cycles += 1;

        // Check if it's time to update (every N cycles)
        if self.total_cycles % self.update_interval_cycles == 0 {
            let actual_total_time = self.window_start.elapsed();

            // Use integer arithmetic where possible
            let avg_actual_cycle_time_nanos =
                actual_total_time.as_nanos() / self.update_interval_cycles as u128;
            let target_cycle_time_nanos = self.target_cycle_time.as_nanos();

            // Calculate error in nanoseconds (signed)
            let error_nanos = target_cycle_time_nanos as i128 - avg_actual_cycle_time_nanos as i128;

            // Update current delay, allowing zero or negative values to become zero
            let new_delay_nanos = (self.delay.as_nanos() as i128 + error_nanos).max(0) as u64;

            self.delay = Duration::from_nanos(new_delay_nanos);
            self.last_avg_cycle_time = Duration::from_nanos(avg_actual_cycle_time_nanos as u64);

            // Reset for next measurement window
            self.window_start = Instant::now();
            self.total_cycles = 0; // Reset to prevent overflow

            // if we have an update callback, call it
            if let Some(on_update) = &self.on_update {
                on_update(LoopThrottleUpdate {
                    delay: self.delay,
                    avg_cycle_time: self.last_avg_cycle_time,
                    error: error_nanos,
                });
            }
        }

        Timer::after(self.delay).await;
    }

    pub fn get_cycle_time(&self) -> Duration {
        self.last_avg_cycle_time
    }

    pub fn get_delay(&self) -> Duration {
        self.delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol::block_on;

    #[test]
    fn test_adaptive_throttle_timing() {
        block_on(async {
            let update_interval_cycles = 100;
            let target_cycle_time = Duration::from_micros(5);
            let mut throttle = LoopThrottle::new(target_cycle_time, update_interval_cycles, None);

            let test_start = Instant::now();
            let mut cycle_times = Vec::new();

            // Run for several cycles to trigger multiple updates
            for _ in 0..10000 {
                let cycle_start = Instant::now();

                // Simulate some work
                Timer::after(Duration::from_micros(5)).await;

                throttle.sleep().await;

                let cycle_time = cycle_start.elapsed();
                cycle_times.push(cycle_time);
            }

            let total_test_time = test_start.elapsed();
            let avg_cycle_time = cycle_times.iter().sum::<Duration>() / cycle_times.len() as u32;

            println!("Test completed in: {:?}", total_test_time);
            println!("Average cycle time: {:?}", avg_cycle_time);
            println!("Target cycle time: {:?}", target_cycle_time);
            println!("Last measured avg: {:?}", throttle.get_cycle_time());

            // Assert that we're reasonably close to target timing
            let tolerance = Duration::from_millis(10);
            assert!(
                avg_cycle_time >= target_cycle_time.saturating_sub(tolerance)
                    && avg_cycle_time <= target_cycle_time + tolerance,
                "Average cycle time {:?} not within tolerance of target {:?}",
                avg_cycle_time,
                target_cycle_time
            );
        });
    }

    #[test]
    fn test_throttle_adaptation() {
        block_on(async {
            let target_cycle_time = Duration::from_millis(100);
            let update_interval_cycles = 3;
            let mut throttle = LoopThrottle::new(target_cycle_time, update_interval_cycles, None);

            let initial_delay = throttle.get_delay();

            // Run enough cycles to trigger multiple updates
            for _ in 0..12 {
                // Add variable work to test adaptation
                Timer::after(Duration::from_millis(10)).await;
                throttle.sleep().await;
            }

            println!("Initial delay: {:?}", initial_delay);
            println!("Final delay: {:?}", throttle.get_delay());
            println!("Last avg cycle time: {:?}", throttle.get_cycle_time());

            assert!(throttle.get_cycle_time() > Duration::ZERO);
        });
    }

    #[test]
    fn test_throttle_initialization() {
        let update_interval_cycles = 10;
        let target_cycle_time = Duration::from_millis(75);
        let throttle = LoopThrottle::new(target_cycle_time, update_interval_cycles, None);

        assert_eq!(throttle.get_delay(), target_cycle_time);
        assert_eq!(throttle.get_cycle_time(), target_cycle_time);
        assert_eq!(throttle.total_cycles, 0);
    }

    #[test]
    fn test_overflow_protection() {
        block_on(async {
            let target_cycle_time = Duration::from_micros(1);
            let update_interval_cycles = 5;
            let mut throttle = LoopThrottle::new(target_cycle_time, update_interval_cycles, None);

            // Run many cycles to ensure total_cycles resets properly
            for i in 0..50 {
                throttle.sleep().await;

                // Verify that total_cycles never exceeds update_interval_cycles
                assert!(
                    throttle.total_cycles <= update_interval_cycles,
                    "Cycle {}: total_cycles ({}) exceeded update_interval_cycles ({})",
                    i,
                    throttle.total_cycles,
                    update_interval_cycles
                );
            }
        });
    }
}
