use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use smol::Timer;

use crate::metrics::csv_writer::{RuntimeSample, append_runtime_sample_csv};
use crate::metrics::io::{NetDevCounters, get_ethercat_iface, read_netdev_counters};
use crate::metrics::jitter::snapshot_machines_jitter;
use crate::metrics::preemption::{get_rt_loop_tid, read_thread_sched_stats};
use crate::metrics::process::ProcessMetrics;
use crate::metrics::state::set_latest_runtime_sample;

/// Configuration for the runtime metrics sampler.
#[derive(Debug, Clone)]
pub struct RuntimeMetricsConfig {
    pub csv_path: String,
    pub interval: Duration,
    pub ethercat_iface: Option<String>,
}

/// Start a background task that periodically samples runtime metrics and
/// appends them to a CSV file.
pub fn spawn_runtime_metrics_sampler(cfg: RuntimeMetricsConfig) {
    smol::spawn(async move {
        let mut last_net: Option<(Instant, NetDevCounters)> = None;
        let mut last_proc: Option<ProcessMetrics> = None;

        loop {
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();

            // 1) Process metrics
            let proc = ProcessMetrics::collect();

            // naively expects values for every second
            let faults_since_last = match last_proc {
                Some(last) => {
                    if proc.minor_faults > last.minor_faults {
                        proc.minor_faults - last.minor_faults
                    } else {
                        0
                    }
                }
                None => 0,
            };

            let mut sample = RuntimeSample::from_process_metrics(
                last_proc.unwrap_or(ProcessMetrics::default()),
                faults_since_last,
                now_ms,
            );
            last_proc = Some(proc);

            // 2) Jitter summary (SIGNED, nanoseconds)
            let jitter_samples = snapshot_machines_jitter();
            if !jitter_samples.is_empty() {
                let mut min = i64::MAX;
                let mut max = i64::MIN;
                let mut sum: i128 = 0;
                let mut count: i128 = 0;

                for s in jitter_samples {
                    let j = s.jitter_ns as i64;
                    min = min.min(j);
                    max = max.max(j);
                    sum += j as i128;
                    count += 1;
                }

                if count > 0 {
                    sample.jitter_min_ns = min;
                    sample.jitter_max_ns = max;
                    sample.jitter_avg_ns = (sum / count) as i64;
                }
            }

            // 3) IO utilization (EtherCAT NIC)
            let iface_name = cfg
                .ethercat_iface
                .clone()
                .or_else(|| get_ethercat_iface().map(|s| s.to_string()));

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
                }
            }

            // 4) Preemption stats (cumulative)
            if let Some(tid) = get_rt_loop_tid() {
                if let Some(stats) = read_thread_sched_stats(tid) {
                    sample.rt_nr_switches = Some(stats.nr_switches);
                    sample.rt_nr_voluntary_switches = Some(stats.nr_voluntary_switches);
                    sample.rt_nr_involuntary_switches = Some(stats.nr_involuntary_switches);
                }
            }

            // 5) Update in-memory state + CSV
            set_latest_runtime_sample(&sample);
            let _ = append_runtime_sample_csv(&cfg.csv_path, &sample);

            Timer::after(cfg.interval).await;
        }
    })
    .detach();
}
