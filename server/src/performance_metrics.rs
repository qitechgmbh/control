use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tracing::info;

/// Configuration for performance metrics collection
const METRICS_WINDOW_SIZE: usize = 1000 * 30; // Keep last 30k measurements
const METRICS_LOG_INTERVAL_SECS: u64 = 30; // Log every 30 seconds

/// Collects and manages EtherCAT performance metrics
pub struct EthercatPerformanceMetrics {
    txrx_times: VecDeque<Duration>,
    loop_times: VecDeque<Duration>,
    pub last_loop_start: Option<Instant>,
    pub last_log_time: Instant,
}

impl Default for EthercatPerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl EthercatPerformanceMetrics {
    /// Creates a new metrics collector
    pub fn new() -> Self {
        Self {
            txrx_times: VecDeque::with_capacity(METRICS_WINDOW_SIZE),
            loop_times: VecDeque::with_capacity(METRICS_WINDOW_SIZE),
            last_loop_start: None,
            last_log_time: Instant::now(),
        }
    }

    /// Records the start of a new cycle
    pub fn cycle_start(&mut self) {
        let now = Instant::now();

        // Record cycle time if we have a previous cycle start
        if let Some(last_start) = self.last_loop_start {
            let cycle_time = now - last_start;
            self.add_cycle_time(cycle_time);
        }

        self.last_loop_start = Some(now);
    }

    /// Adds a tx_rx time measurement
    pub fn add_txrx_time(&mut self, duration: Duration) {
        if self.txrx_times.len() >= METRICS_WINDOW_SIZE {
            self.txrx_times.pop_front();
        }
        self.txrx_times.push_back(duration);
    }

    /// Adds a cycle time measurement
    fn add_cycle_time(&mut self, duration: Duration) {
        if self.loop_times.len() >= METRICS_WINDOW_SIZE {
            self.loop_times.pop_front();
        }
        self.loop_times.push_back(duration);
    }

    /// Logs metrics if enough time has passed
    pub fn maybe_log_metrics(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_log_time).as_secs() >= METRICS_LOG_INTERVAL_SECS {
            self.log_metrics();
            self.last_log_time = now;
        }
    }

    /// Logs the current metrics
    fn log_metrics(&self) {
        if !self.txrx_times.is_empty() {
            let txrx_stats = calculate_stats(&self.txrx_times);
            info!(
                "TxRx metrics - avg: {:.3}ms, 99.99th: {:.3}ms, stddev: {:.3}ms, samples: {}",
                txrx_stats.average_ms,
                txrx_stats.percentile_9999_ms,
                txrx_stats.stddev_ms,
                self.txrx_times.len()
            );
        }

        if !self.loop_times.is_empty() {
            let cycle_stats = calculate_stats(&self.loop_times);
            info!(
                "Loop metrics - avg: {:.3}ms, 99.99th: {:.3}ms, stddev: {:.3}ms, samples: {}",
                cycle_stats.average_ms,
                cycle_stats.percentile_9999_ms,
                cycle_stats.stddev_ms,
                self.loop_times.len()
            );
        }
    }
}

/// Statistical metrics for a set of duration measurements
#[derive(Debug)]
struct MetricsStats {
    average_ms: f64,
    percentile_9999_ms: f64,
    stddev_ms: f64,
}

/// Calculates statistical metrics for a collection of durations
fn calculate_stats(durations: &VecDeque<Duration>) -> MetricsStats {
    if durations.is_empty() {
        return MetricsStats {
            average_ms: 0.0,
            percentile_9999_ms: 0.0,
            stddev_ms: 0.0,
        };
    }

    // Convert to microseconds for calculation to avoid precision loss
    let values_us: Vec<f64> = durations
        .iter()
        .map(|d| d.as_nanos() as f64 / 1000.0)
        .collect();

    let count = values_us.len() as f64;
    let sum: f64 = values_us.iter().sum();
    let average_us = sum / count;

    // Calculate standard deviation
    let variance = values_us
        .iter()
        .map(|x| (x - average_us).powi(2))
        .sum::<f64>()
        / count;
    let stddev_us = variance.sqrt();

    // Calculate 99.99th percentile
    let mut sorted_values = values_us;
    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let percentile_index = ((count * 0.9999) as usize).min(sorted_values.len() - 1);
    let percentile_9999_us = sorted_values[percentile_index];

    MetricsStats {
        average_ms: average_us / 1000.0,
        percentile_9999_ms: percentile_9999_us / 1000.0,
        stddev_ms: stddev_us / 1000.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_stats_empty() {
        let durations = VecDeque::new();
        let stats = calculate_stats(&durations);
        assert_eq!(stats.average_ms, 0.0);
        assert_eq!(stats.percentile_9999_ms, 0.0);
        assert_eq!(stats.stddev_ms, 0.0);
    }

    #[test]
    fn test_calculate_stats_single_value() {
        let mut durations = VecDeque::new();
        durations.push_back(Duration::from_millis(10));
        let stats = calculate_stats(&durations);
        assert!((stats.average_ms - 10.0).abs() < 0.001);
        assert!((stats.percentile_9999_ms - 10.0).abs() < 0.001);
        assert_eq!(stats.stddev_ms, 0.0);
    }

    #[test]
    fn test_metrics_collection() {
        let mut metrics = EthercatPerformanceMetrics::new();

        // Record some measurements
        metrics.add_txrx_time(Duration::from_millis(1));
        metrics.add_txrx_time(Duration::from_millis(2));
        metrics.add_txrx_time(Duration::from_millis(3));

        assert_eq!(metrics.txrx_times.len(), 3);
    }
}
