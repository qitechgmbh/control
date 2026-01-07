use libc;
use std::fs;
use std::sync::OnceLock;

/// Process-level metrics for the server process.
///
/// All values are cumulative since process start.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessMetrics {
    pub rss_bytes: u64,
    pub cpu_time_seconds: f64,
    pub minor_faults: u64,
    pub major_faults: u64,
}

// Cache system constants once
static PAGE_SIZE: OnceLock<u64> = OnceLock::new();
static CLK_TCK: OnceLock<f64> = OnceLock::new();

fn page_size() -> u64 {
    *PAGE_SIZE.get_or_init(|| unsafe { libc::sysconf(libc::_SC_PAGESIZE).max(1) as u64 })
}

fn clk_tck() -> f64 {
    *CLK_TCK.get_or_init(|| unsafe { libc::sysconf(libc::_SC_CLK_TCK).max(1) as f64 })
}

impl ProcessMetrics {
    /// Collect current process metrics from /proc.
    ///
    /// Linux-specific; on other platforms returns zeroed metrics.
    pub fn collect() -> ProcessMetrics {
        #[cfg(target_os = "linux")]
        {
            let mut rss_bytes = 0;
            let mut cpu_time_seconds = 0.0;
            let mut minor_faults = 0;
            let mut major_faults = 0;

            // ---- RSS from /proc/self/statm ----
            if let Ok(statm) = fs::read_to_string("/proc/self/statm") {
                let mut parts = statm.split_whitespace();
                let _ = parts.next(); // total size
                if let Some(resident_pages_str) = parts.next() {
                    if let Ok(resident_pages) = resident_pages_str.parse::<u64>() {
                        rss_bytes = resident_pages.saturating_mul(page_size());
                    }
                }
            }

            // ---- CPU time + faults from /proc/self/stat ----
            if let Ok(stat) = fs::read_to_string("/proc/self/stat") {
                // Find the end of the "(comm)" field safely
                if let Some(end_comm) = stat.rfind(") ") {
                    let after = &stat[end_comm + 2..];
                    let parts: Vec<&str> = after.split_whitespace().collect();

                    // Indices relative to AFTER comm:
                    // 7: minflt, 9: majflt, 11: utime, 12: stime
                    if parts.len() > 12 {
                        minor_faults = parts[7].parse().unwrap_or(0);
                        major_faults = parts[9].parse().unwrap_or(0);

                        let utime: u64 = parts[11].parse().unwrap_or(0);
                        let stime: u64 = parts[12].parse().unwrap_or(0);

                        cpu_time_seconds = (utime + stime) as f64 / clk_tck();
                    }
                }
            }

            ProcessMetrics {
                rss_bytes,
                cpu_time_seconds,
                minor_faults,
                major_faults,
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            ProcessMetrics {
                rss_bytes: 0,
                cpu_time_seconds: 0.0,
                minor_faults: 0,
                major_faults: 0,
            }
        }
    }
}
