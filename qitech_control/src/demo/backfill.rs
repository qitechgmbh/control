//! Fills the demo machines' last 30 days of measurements inside ClickHouse, at the
//! same 32 Hz the live data has, so the graphs can be tested against a full database.
//!
//! Only missing time is filled: the range before the oldest stored demo row and the gap
//! between the newest stored row and this start (the backend was off). Rows are made by
//! `INSERT … SELECT … FROM numbers()` inside ClickHouse, one series and day per insert,
//! newest day first, so each insert stays far below the server's `max_execution_time`.

use std::sync::Mutex;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use chrono::DateTime;
use chrono::Utc;
use futures_util::StreamExt;
use futures_util::stream;
use qitech_framework::ConfigPropertyEvent;
use qitech_framework::ConfigPropertyEventRecord;
use qitech_framework::Constraints;
use qitech_framework::ScalarValue;
use qitech_framework_core::report::OperationCapability;
use serde::Serialize;

use crate::api::db_tables::machine_config_property_records;
use crate::api::db_tables::machine_measurements;
use crate::demo::line;
use crate::demo::line::Initial;
use crate::demo::signal;

/// History kept by the TTL of `machine_measurements`.
const RETENTION_MS: i64 = 30 * DAY_MS;
const DAY_MS: i64 = 24 * 3600 * 1000;
/// Live data is exported at 32 Hz: one row every 31.25 ms, i.e. 4 rows per 125 ms.
const ROWS_PER_125_MS: i64 = 4;
/// Gaps shorter than this are left alone (a restart takes a few seconds anyway).
const MIN_GAP_MS: i64 = 60_000;
/// Compressed bytes per measurement row, measured on the demo data and rounded up.
/// Only used to keep the backfill from filling the disk.
const BYTES_PER_ROW_ESTIMATE: f64 = 6.0;
/// Share of the free disk space the backfill may use.
const MAX_DISK_SHARE: f64 = 0.6;

#[derive(Debug, Default)]
pub struct Progress {
    state: AtomicU8,
    done: AtomicU64,
    total: AtomicU64,
    rows: AtomicU64,
    error: Mutex<Option<String>>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct Status {
    pub state: State,
    /// Inserts (one series, one day) done and planned.
    pub done: u64,
    pub total: u64,
    /// Rows inserted so far.
    pub rows: u64,
    pub error: Option<String>,
}

impl Progress {
    pub fn status(&self) -> Status {
        Status {
            state: match self.state.load(Ordering::Relaxed) {
                1 => State::Running,
                2 => State::Done,
                3 => State::Failed,
                _ => State::Pending,
            },
            done: self.done.load(Ordering::Relaxed),
            total: self.total.load(Ordering::Relaxed),
            rows: self.rows.load(Ordering::Relaxed),
            error: self.error.lock().unwrap_or_else(|p| p.into_inner()).clone(),
        }
    }

    fn set_state(&self, state: State) {
        let v = match state {
            State::Pending => 0,
            State::Running => 1,
            State::Done => 2,
            State::Failed => 3,
        };
        self.state.store(v, Ordering::Relaxed);
    }

    fn fail(&self, error: String) {
        tracing::error!(%error, "demo backfill failed");
        *self.error.lock().unwrap_or_else(|p| p.into_inner()) = Some(error);
        self.set_state(State::Failed);
    }
}

/// One insert: one measurement of one machine over `[from_ms, to_ms)` within a UTC day.
struct Job {
    identity: u64,
    name: &'static str,
    expr: String,
    from_ms: i64,
    to_ms: i64,
}

impl Job {
    fn rows(&self) -> u64 {
        // rows at from + floor(i * 31.25 ms) that fall before `to`
        (((self.to_ms - self.from_ms) * ROWS_PER_125_MS) as u64).div_ceil(125)
    }
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct Coverage {
    n: u64,
    lo: i64,
    hi: i64,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct Free {
    free: u64,
}

/// Runs the backfill. `start_ms` is when the live demo data began; nothing at or after
/// it is written.
pub async fn run(db: clickhouse::Client, progress: &Progress, start_ms: i64) {
    progress.set_state(State::Running);
    if let Err(e) = run_inner(&db, progress, start_ms).await {
        progress.fail(e.to_string());
        return;
    }
    progress.set_state(State::Done);
    let status = progress.status();
    tracing::info!(
        rows = status.rows,
        inserts = status.done,
        "demo backfill done"
    );
}

async fn run_inner(
    db: &clickhouse::Client,
    progress: &Progress,
    start_ms: i64,
) -> Result<(), clickhouse::error::Error> {
    let free: Free = db
        .query("SELECT toUInt64(sum(free_space)) AS free FROM system.disks WHERE name = 'default'")
        .fetch_one()
        .await?;

    // --- rows the full line writes per day, to cap the backfill at the free space ---
    let series: usize = line::machines().iter().map(|m| m.measurements.len()).sum();
    let bytes_per_day =
        series as f64 * (DAY_MS * ROWS_PER_125_MS / 125) as f64 * BYTES_PER_ROW_ESTIMATE;
    let days_fitting = if free.free == 0 {
        tracing::warn!("free disk space unknown, not limiting the demo backfill");
        30
    } else {
        (free.free as f64 * MAX_DISK_SHARE / bytes_per_day).floor() as i64
    };
    let oldest_ms = start_ms - RETENTION_MS.min(days_fitting * DAY_MS);
    if days_fitting < 30 {
        tracing::warn!(
            days_fitting,
            free_bytes = free.free,
            "not enough free disk space for 30 days of demo data, backfilling fewer days"
        );
    }

    let mut jobs = Vec::new();
    let mut registrations = Vec::new();

    for def in line::machines() {
        let identity = def.ident().to_u64();
        let coverage: Coverage = db
            .query(
                "SELECT count() AS n,
                        toUnixTimestamp64Milli(min(timestamp)) AS lo,
                        toUnixTimestamp64Milli(max(timestamp)) AS hi
                 FROM machine_measurements
                 WHERE identity = ? AND timestamp < fromUnixTimestamp64Milli(?)",
            )
            .bind(identity)
            .bind(start_ms)
            .fetch_one()
            .await?;

        // --- missing ranges: before the oldest row, and between the newest and now ---
        let mut ranges = Vec::new();
        if coverage.n == 0 {
            ranges.push((oldest_ms, start_ms));
        } else {
            if coverage.lo - oldest_ms > MIN_GAP_MS {
                ranges.push((oldest_ms, coverage.lo));
            }
            if start_ms - coverage.hi > MIN_GAP_MS {
                ranges.push((coverage.hi + 32, start_ms));
            }
        }
        let Some(&(head_ms, _)) = ranges.first() else {
            continue;
        };

        // --- targets in effect from the start of the history, for the graphs' target lines ---
        if coverage.n == 0 || head_ms < coverage.lo {
            registrations.extend(def.config.iter().filter_map(|(path, initial)| {
                let record = ConfigPropertyEventRecord {
                    timestamp: DateTime::<Utc>::from_timestamp_millis(head_ms).unwrap_or_default(),
                    machine: def.ident(),
                    path: (*path).to_owned(),
                    event: ConfigPropertyEvent::Registered {
                        default: match *initial {
                            Initial::Float(v) => ScalarValue::Float(v),
                            Initial::Bool(b) => ScalarValue::Boolean(b),
                            Initial::Enum(v) => ScalarValue::Enum(crate::demo::variant_ident(v)),
                        },
                        capability: OperationCapability::Allowed,
                        constraints: Constraints::None,
                    },
                };
                machine_config_property_records::Row::try_from(&record)
                    .inspect_err(|e| tracing::error!("demo backfill: skipping {e}"))
                    .ok()
            }));
        }

        for (from_ms, to_ms) in ranges {
            for (day_from, day_to) in split_at_days(from_ms, to_ms) {
                for (i, (name, signal)) in def.measurements.iter().enumerate() {
                    let base = match signal {
                        signal::Signal::Wave { base, .. } => match base {
                            signal::Base::Const(v) => *v,
                            signal::Base::Config { path, offset } => {
                                def.config_float(path) + offset
                            }
                        },
                        _ => 0.0,
                    };
                    jobs.push(Job {
                        identity,
                        name,
                        expr: signal.sql(base, signal::phase(i)),
                        from_ms: day_from,
                        to_ms: day_to,
                    });
                }
            }
        }
    }

    if !registrations.is_empty() {
        let mut insert = db
            .insert::<machine_config_property_records::Row>(
                machine_config_property_records::TABLE_NAME,
            )
            .await?;
        for row in &registrations {
            insert.write(row).await?;
        }
        insert.end().await?;
    }

    // newest data first: the graphs open on the recent past
    jobs.sort_by_key(|job| std::cmp::Reverse(job.from_ms));
    progress.total.store(jobs.len() as u64, Ordering::Relaxed);
    let total_rows: u64 = jobs.iter().map(Job::rows).sum();
    tracing::info!(
        inserts = jobs.len(),
        rows = total_rows,
        estimated_bytes = (total_rows as f64 * BYTES_PER_ROW_ESTIMATE) as u64,
        "demo backfill started"
    );

    // Leave cores for the runtime, the UI and the live inserts.
    let concurrency = std::thread::available_parallelism().map_or(1, |n| (n.get() / 4).clamp(1, 4));

    let mut results = stream::iter(jobs)
        .map(|job| async move {
            let rows = job.rows();
            insert_series(db, &job).await.map(|()| rows)
        })
        .buffer_unordered(concurrency);

    while let Some(result) = results.next().await {
        let rows = result?;
        progress.rows.fetch_add(rows, Ordering::Relaxed);
        let done = progress.done.fetch_add(1, Ordering::Relaxed) + 1;
        if done % 100 == 0 {
            tracing::info!(
                done,
                total = progress.total.load(Ordering::Relaxed),
                "demo backfill progress"
            );
        }
    }

    Ok(())
}

async fn insert_series(db: &clickhouse::Client, job: &Job) -> Result<(), clickhouse::error::Error> {
    // `t` (seconds since the signal epoch) is what the signal expression reads.
    let sql = format!(
        "INSERT INTO {table} (timestamp, identity, name, value)
         SELECT fromUnixTimestamp64Milli(ms), ?, ?, v
         FROM
         (
             SELECT ms, {expr} AS v
             FROM
             (
                 SELECT toInt64(? + intDiv(number * 125, {ROWS_PER_125_MS})) AS ms,
                        (ms - {epoch}) / 1000.0 AS t
                 FROM numbers(?)
             )
         )",
        table = machine_measurements::TABLE_NAME,
        expr = job.expr,
        epoch = signal::EPOCH_MS,
    );
    db.query(&sql)
        .bind(job.identity)
        .bind(job.name)
        .bind(job.from_ms)
        .bind(job.rows())
        .execute()
        .await
}

/// Splits `[from, to)` at UTC midnights, so each insert writes into one daily partition.
fn split_at_days(from_ms: i64, to_ms: i64) -> Vec<(i64, i64)> {
    let mut out = Vec::new();
    let mut from = from_ms;
    while from < to_ms {
        let midnight = (from.div_euclid(DAY_MS) + 1) * DAY_MS;
        let to = midnight.min(to_ms);
        out.push((from, to));
        from = to;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_at_midnight() {
        let from = 3 * DAY_MS - 1000;
        let to = 4 * DAY_MS + 5000;
        assert_eq!(
            split_at_days(from, to),
            vec![
                (from, 3 * DAY_MS),
                (3 * DAY_MS, 4 * DAY_MS),
                (4 * DAY_MS, to)
            ]
        );
    }

    #[test]
    fn one_day_has_32_hz_rows() {
        let job = Job {
            identity: 0,
            name: "x",
            expr: String::new(),
            from_ms: 0,
            to_ms: DAY_MS,
        };
        assert_eq!(job.rows(), 86_400 * 32);
    }
}
