use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::metrics::process::ProcessMetrics;

/// One row of runtime metrics for CSV export.
///
/// All fields are "instantaneous" samples at `timestamp_ms`.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeSample {
    pub timestamp_ms: u128,

    // process metrics
    pub rss_bytes: u64,
    pub cpu_time_seconds: f64,
    pub minor_faults: u64,
    pub major_faults: u64,

    // jitter summary (machines loop), in nanoseconds
    pub jitter_min_ns: u64,
    pub jitter_avg_ns: u64,
    pub jitter_max_ns: u64,

    // network IO (EtherCAT NIC) byte rates over last interval
    pub rx_rate_bytes_per_sec: f64,
    pub tx_rate_bytes_per_sec: f64,

    // optional: preemption stats (current values)
    pub rt_nr_switches: Option<u64>,
    pub rt_nr_voluntary_switches: Option<u64>,
    pub rt_nr_involuntary_switches: Option<u64>,
}

impl RuntimeSample {
    /// Create a sample with only process metrics filled; other fields can be
    /// filled by the sampler before writing.
    pub fn from_process_metrics(m: ProcessMetrics, timestamp_ms: u128) -> Self {
        Self {
            timestamp_ms,
            rss_bytes: m.rss_bytes,
            cpu_time_seconds: m.cpu_time_seconds,
            minor_faults: m.minor_faults,
            major_faults: m.major_faults,

            jitter_min_ns: 0,
            jitter_avg_ns: 0,
            jitter_max_ns: 0,

            rx_rate_bytes_per_sec: 0.0,
            tx_rate_bytes_per_sec: 0.0,

            rt_nr_switches: None,
            rt_nr_voluntary_switches: None,
            rt_nr_involuntary_switches: None,
        }
    }
}

/// Append a header row if the file is new.
fn write_header<W: Write>(mut w: W) -> std::io::Result<()> {
    writeln!(
        w,
        "timestamp_ms,\
         rss_bytes,\
         cpu_time_seconds,\
         minor_faults,\
         major_faults,\
         jitter_min_ns,\
         jitter_avg_ns,\
         jitter_max_ns,\
         rx_rate_bytes_per_sec,\
         tx_rate_bytes_per_sec,\
         rt_nr_switches,\
         rt_nr_voluntary_switches,\
         rt_nr_involuntary_switches"
    )
}

/// Append one sample as a CSV row. Creates the file and header if needed.
pub fn append_runtime_sample_csv<P: AsRef<Path>>(
    path: P,
    sample: &RuntimeSample,
) -> std::io::Result<()> {
    let path = path.as_ref();
    let file_existed = path.exists();

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    let mut writer = BufWriter::new(file);

    if !file_existed {
        write_header(&mut writer)?;
    }

    // Use u64::MAX for "None" preemption stats to keep the column numeric.
    let nr_switches = sample.rt_nr_switches.unwrap_or(u64::MAX);
    let nr_vol = sample.rt_nr_voluntary_switches.unwrap_or(u64::MAX);
    let nr_invol = sample.rt_nr_involuntary_switches.unwrap_or(u64::MAX);

    writeln!(
        writer,
        "{},{},{:.6},{},{},\
         {},{},{} ,\
         {:.6},{:.6},\
         {},{},{}",
        sample.timestamp_ms,
        sample.rss_bytes,
        sample.cpu_time_seconds,
        sample.minor_faults,
        sample.major_faults,
        sample.jitter_min_ns,
        sample.jitter_avg_ns,
        sample.jitter_max_ns,
        sample.rx_rate_bytes_per_sec,
        sample.tx_rate_bytes_per_sec,
        nr_switches,
        nr_vol,
        nr_invol,
    )?;

    writer.flush()?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_header_and_row() {
        // Generate a unique filename in the OS temp dir.
        let tmp_dir = std::env::temp_dir();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = tmp_dir.join(format!("runtime_metrics_test_{ts}.csv"));

        let m = ProcessMetrics {
            rss_bytes: 123,
            cpu_time_seconds: 1.23,
            minor_faults: 10,
            major_faults: 1,
        };
        let mut s = RuntimeSample::from_process_metrics(m, 42);
        s.jitter_min_ns = 1;
        s.jitter_avg_ns = 2;
        s.jitter_max_ns = 3;
        s.rx_rate_bytes_per_sec = 100.0;
        s.tx_rate_bytes_per_sec = 200.0;
        s.rt_nr_switches = Some(5);
        s.rt_nr_voluntary_switches = Some(3);
        s.rt_nr_involuntary_switches = Some(2);

        append_runtime_sample_csv(&path, &s).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = contents.lines().collect();
        assert!(lines.len() >= 2);
        assert!(lines[0].starts_with("timestamp_ms,"));

        // Best-effort cleanup.
        let _ = fs::remove_file(&path);
    }
}