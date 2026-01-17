use std::sync::Arc;

use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::SharedState;
use crate::metrics::process::ProcessMetrics;
use crate::metrics::state::get_latest_runtime_sample;

/// Process-level metrics exposed over the REST API.
///
/// Counters are cumulative since process start.
#[derive(Debug, Serialize)]
pub struct ProcessMetricsResponse {
    pub rss_bytes: u64,
    pub cpu_time_seconds: f64,
    pub minor_faults: u64,
    pub major_faults: u64,
}

/// Snapshot of runtime metrics for the frontend.
///
/// This is a thin view over the latest RuntimeSample.
#[derive(Debug, Serialize)]
pub struct RuntimeMetricsResponse {
    pub timestamp_ms: u128,

    // process metrics
    pub rss_bytes: u64,
    pub cpu_time_seconds: f64,
    pub minor_faults: u64,
    pub major_faults: u64,

    // jitter (SIGNED nanoseconds: negative = early, positive = late)
    pub jitter_min_ns: i64,
    pub jitter_avg_ns: i64,
    pub jitter_max_ns: i64,

    // network IO
    pub rx_rate_bytes_per_sec: f64,
    pub tx_rate_bytes_per_sec: f64,

    // preemption stats (cumulative)
    pub rt_nr_switches: Option<u64>,
    pub rt_nr_voluntary_switches: Option<u64>,
    pub rt_nr_involuntary_switches: Option<u64>,
}

async fn get_process_metrics() -> Json<ProcessMetricsResponse> {
    let m = ProcessMetrics::collect();
    Json(ProcessMetricsResponse {
        rss_bytes: m.rss_bytes,
        cpu_time_seconds: m.cpu_time_seconds,
        minor_faults: m.minor_faults,
        major_faults: m.major_faults,
    })
}

async fn get_runtime_metrics_latest() -> Json<Option<RuntimeMetricsResponse>> {
    let opt = get_latest_runtime_sample().map(|s| RuntimeMetricsResponse {
        timestamp_ms: s.timestamp_ms,

        rss_bytes: s.rss_bytes,
        cpu_time_seconds: s.cpu_time_seconds,
        minor_faults: s.minor_faults_per_second,
        major_faults: s.major_faults,

        jitter_min_ns: s.jitter_min_ns,
        jitter_avg_ns: s.jitter_avg_ns,
        jitter_max_ns: s.jitter_max_ns,

        rx_rate_bytes_per_sec: s.rx_rate_bytes_per_sec,
        tx_rate_bytes_per_sec: s.tx_rate_bytes_per_sec,

        rt_nr_switches: s.rt_nr_switches,
        rt_nr_voluntary_switches: s.rt_nr_voluntary_switches,
        rt_nr_involuntary_switches: s.rt_nr_involuntary_switches,
    });

    Json(opt)
}

/// Router for metrics-related REST endpoints.
///
/// Mounted under `/api/v1/metrics`.
pub fn metrics_router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/process/metrics", get(get_process_metrics))
        .route("/runtime/latest", get(get_runtime_metrics_latest))
}
