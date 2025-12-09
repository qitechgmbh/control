use std::fs;
use libc;

/// Process-level metrics for the server process.
///
/// - `rss_bytes`: resident set size (bytes)
/// - `cpu_time_seconds`: user + system CPU time (seconds, cumulative)
/// - `minor_faults`: page faults that did not require disk I/O (cumulative)
/// - `major_faults`: page faults that required disk I/O (cumulative)
#[derive(Debug, Clone, Copy)]
pub struct ProcessMetrics {
    pub rss_bytes: u64,
    pub cpu_time_seconds: f64,
    pub minor_faults: u64,
    pub major_faults: u64,
}

impl ProcessMetrics {
    /// Collect current process metrics from /proc.
    ///
    /// Linux-specific; on other platforms returns zeroed metrics.
    pub fn collect() -> ProcessMetrics {
        #[cfg(target_os = "linux")]
        {
            let mut rss_bytes = 0u64;
            let mut cpu_time_seconds = 0.0f64;
            let mut minor_faults = 0u64;
            let mut major_faults = 0u64;

            // RSS from /proc/self/statm
            if let Ok(statm) = fs::read_to_string("/proc/self/statm") {
                let mut parts = statm.split_whitespace();
                let _ = parts.next(); // total size
                if let Some(resident_pages_str) = parts.next() {
                    if let Ok(resident_pages) = resident_pages_str.parse::<u64>() {
                        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 };
                        rss_bytes = resident_pages.saturating_mul(page_size);
                    }
                }
            }

            // CPU time and page faults from /proc/self/stat
            if let Ok(stat) = fs::read_to_string("/proc/self/stat") {
                let parts: Vec<_> = stat.split_whitespace().collect();
                // Format (1-based indices):
                // 10: minflt, 12: majflt, 14: utime, 15: stime
                if parts.len() > 15 {
                    if let Ok(v) = parts[9].parse::<u64>() {
                        minor_faults = v;
                    }
                    if let Ok(v) = parts[11].parse::<u64>() {
                        major_faults = v;
                    }
                    if let (Ok(utime), Ok(stime)) =
                        (parts[13].parse::<u64>(), parts[14].parse::<u64>())
                    {
                        let ticks_per_sec =
                            unsafe { libc::sysconf(libc::_SC_CLK_TCK) as f64 }.max(1.0);
                        cpu_time_seconds = (utime + stime) as f64 / ticks_per_sec;
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