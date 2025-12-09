use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use smol::Timer;

use crate::metrics::csv_writer::{append_runtime_sample_csv, RuntimeSample};
use crate::metrics::io::{read_netdev_counters, NetDevCounters, get_ethercat_iface};
use crate::metrics::jitter::snapshot_machines_jitter;
use crate::metrics::process::ProcessMetrics;
use crate::metrics::preemption::{read_thread_sched_stats, get_rt_loop_tid};
use crate::metrics::state::set_latest_runtime_sample; 

/// Configuration for the runtime metrics sampler.
#[derive(Debug, Clone)]
pub struct RuntimeMetricsConfig {
    /// Target CSV file path.
    pub csv_path: String,
    /// Sampling interval.
    pub interval: Duration,
    /// Optional EtherCAT NIC interface name (e.g. "enp1s0").
    pub ethercat_iface: Option<String>,

}

/// Start a background task that periodically samples runtime metrics and
/// appends them to a CSV file.
///
/// This feeds offline analysis / graphing (e.g. Python).
pub fn spawn_runtime_metrics_sampler(cfg: RuntimeMetricsConfig) {
    smol::spawn(async move {
        let mut last_net: Option<(Instant, NetDevCounters)> = None;

        loop {
            let now = SystemTime::now();
            let now_ms = now
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();

            // 1) Process metrics
            let proc = ProcessMetrics::collect();
            let mut sample = RuntimeSample::from_process_metrics(proc, now_ms);

            // 2) Jitter summary (Machines loop)
            let jitter_samples = snapshot_machines_jitter();
            if !jitter_samples.is_empty() {
                let mut min = u64::MAX;
                let mut max = 0u64;
                let mut sum = 0u128;
                let mut count = 0u128;

                for s in jitter_samples {
                    if s.jitter_ns == 0 {
                        continue;
                    }
                    min = min.min(s.jitter_ns);
                    max = max.max(s.jitter_ns);
                    sum += s.jitter_ns as u128;
                    count += 1;
                }

                if count > 0 {
                    sample.jitter_min_ns = min;
                    sample.jitter_max_ns = max;
                    sample.jitter_avg_ns = (sum / count) as u64;
                }
            }

            // 3) IO utilization (EtherCAT NIC)
            let iface_name: Option<String> = if let Some(explicit) = &cfg.ethercat_iface {
                Some(explicit.clone())
            } else if let Some(discovered) = get_ethercat_iface() {
                Some(discovered.to_string())
            } else {
                None
            };

            if let Some(iface) = iface_name.as_deref() {
                let now_inst = Instant::now();
                if let Some(curr) = read_netdev_counters(iface) {
                    if let Some((prev_t, prev)) = last_net {
                        let dt = now_inst.saturating_duration_since(prev_t);
                        let dt_s = dt.as_secs_f64().max(1e-6);
                        sample.rx_rate_bytes_per_sec =
                            (curr.rx_bytes.saturating_sub(prev.rx_bytes)) as f64 / dt_s;
                        sample.tx_rate_bytes_per_sec =
                            (curr.tx_bytes.saturating_sub(prev.tx_bytes)) as f64 / dt_s;
                    }
                    last_net = Some((now_inst, curr));
                } else {
                    tracing::warn!(
                        "runtime metrics: could not read netdev counters for iface={iface}"
                    );
                }
            }

            // 4) Preemption stats (still optional)
            if let Some(tid) = get_rt_loop_tid() {
                if let Some(stats) = read_thread_sched_stats(tid) {
                    sample.rt_nr_switches = Some(stats.nr_switches);
                    sample.rt_nr_voluntary_switches =
                        Some(stats.nr_voluntary_switches);
                    sample.rt_nr_involuntary_switches =
                        Some(stats.nr_involuntary_switches);
                }
            }
            // 4.5) Update in‑memory latest (for REST / frontend)
            set_latest_runtime_sample(&sample);
             // 5) Write to CSV
            if let Err(e) = append_runtime_sample_csv(&cfg.csv_path, &sample) {
                tracing::warn!(
                    "runtime metrics: failed to append CSV row to {}: {e}",
                    cfg.csv_path
                );
            }

            // periodic delay using smol
            Timer::after(cfg.interval).await;
        }
    })
    .detach();
}