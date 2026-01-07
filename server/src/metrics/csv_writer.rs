use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::metrics::process::ProcessMetrics;

/// One row of runtime metrics for CSV export.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeSample {
    pub timestamp_ms: u128,

    // process metrics
    pub rss_bytes: u64,
    pub cpu_time_seconds: f64,
    pub minor_faults: u64,
    pub minor_faults_per_second: u64,
    pub major_faults: u64,

    // jitter (signed nanoseconds)
    pub jitter_min_ns: i64,
    pub jitter_avg_ns: i64,
    pub jitter_max_ns: i64,

    // RT loop CPU time (cumulative seconds)
    pub rt_loop_cpu_time_seconds: Option<f64>,

    // network IO
    pub rx_rate_bytes_per_sec: f64,
    pub tx_rate_bytes_per_sec: f64,

    // preemption stats
    pub rt_nr_switches: Option<u64>,
    pub rt_nr_voluntary_switches: Option<u64>,
    pub rt_nr_involuntary_switches: Option<u64>,
}

impl RuntimeSample {
    pub fn from_process_metrics(
        m: ProcessMetrics,
        mfaults_per_second: u64,
        timestamp_ms: u128,
    ) -> Self {
        Self {
            timestamp_ms,
            rss_bytes: m.rss_bytes,
            cpu_time_seconds: m.cpu_time_seconds,
            minor_faults: m.minor_faults,
            major_faults: m.major_faults,
            minor_faults_per_second: mfaults_per_second,

            jitter_min_ns: 0,
            jitter_avg_ns: 0,
            jitter_max_ns: 0,

            rt_loop_cpu_time_seconds: None,

            rx_rate_bytes_per_sec: 0.0,
            tx_rate_bytes_per_sec: 0.0,

            rt_nr_switches: None,
            rt_nr_voluntary_switches: None,
            rt_nr_involuntary_switches: None,
        }
    }
}

fn write_header<W: Write>(mut w: W) -> std::io::Result<()> {
    writeln!(
        w,
        "timestamp_ms,\
         rss_bytes,\
         process_cpu_time_s,\
         minor_faults,\
         major_faults,\
         jitter_min_ns,\
         jitter_avg_ns,\
         jitter_max_ns,\
         rt_loop_cpu_time_s,\
         rx_bytes_per_s,\
         tx_bytes_per_s,\
         rt_nr_switches,\
         rt_nr_voluntary_switches,\
         rt_nr_involuntary_switches"
    )
}

fn opt_u64(v: Option<u64>) -> String {
    v.map(|x| x.to_string()).unwrap_or_default()
}

fn opt_f64(v: Option<f64>) -> String {
    v.map(|x| format!("{:.6}", x)).unwrap_or_default()
}

pub fn append_runtime_sample_csv<P: AsRef<Path>>(
    path: P,
    sample: &RuntimeSample,
) -> std::io::Result<()> {
    let path = path.as_ref();
    let file_existed = path.exists();

    let file = OpenOptions::new().create(true).append(true).open(path)?;

    let mut writer = BufWriter::new(file);

    if !file_existed {
        write_header(&mut writer)?;
    }

    writeln!(
        writer,
        "{},{},{:.6},{},{},\
         {},{},{} ,\
         {},\
         {:.3},{:.3},\
         {},{},{}",
        sample.timestamp_ms,
        sample.rss_bytes,
        sample.cpu_time_seconds,
        sample.minor_faults,
        sample.major_faults,
        sample.jitter_min_ns,
        sample.jitter_avg_ns,
        sample.jitter_max_ns,
        opt_f64(sample.rt_loop_cpu_time_seconds),
        sample.rx_rate_bytes_per_sec,
        sample.tx_rate_bytes_per_sec,
        opt_u64(sample.rt_nr_switches),
        opt_u64(sample.rt_nr_voluntary_switches),
        opt_u64(sample.rt_nr_involuntary_switches),
    )?;

    writer.flush()?;
    Ok(())
}
