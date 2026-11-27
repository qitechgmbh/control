//! `POST /api/v1/machine/history/export`: measurement history of a time range as one
//! wide table (CSV or Parquet), streamed from ClickHouse straight into the response.
//!
//! One row per sample time (measurements of one runtime tick share a timestamp) or, with
//! `bucket_ms`, one row per epoch-aligned bucket with mean, min and max per measurement.
//! Targets (config properties) become step columns: the accepted value in effect at the
//! row's time. ClickHouse formats the output itself, so the server never holds the table
//! in memory, whatever the range.

use std::collections::HashSet;

use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::Response;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::Response as AxumResponse;
use clickhouse::sql::Identifier;
use qitech_framework::MachineInstanceIdentification;
use serde::Deserialize;

use super::history::MAX_NAMES;
use super::history::fetch_targets;
use crate::api::legacy::LegacyApiState;
use crate::api::legacy::response_util::ResponseUtil;
use crate::api::legacy::types::LegacyMachineIdentificationUnique;

/// Smallest bucket width accepted (finer than the runtime tick makes no sense).
const MIN_BUCKET_MS: i64 = 10;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Csv,
    Parquet,
}

impl Format {
    fn clickhouse_format(self) -> &'static str {
        match self {
            Format::Csv => "CSVWithNames",
            Format::Parquet => "Parquet",
        }
    }

    fn content_type(self) -> &'static str {
        match self {
            Format::Csv => "text/csv; charset=utf-8",
            Format::Parquet => "application/vnd.apache.parquet",
        }
    }
}

/// One exported column: a measurement or config path and its column header.
#[derive(Debug, Deserialize)]
pub struct Column {
    /// Measurement path (`heating.nozzle.temperature`) or config path (`screw.target_rpm`).
    pub name: String,
    /// Column header, e.g. `Nozzle Temperature [°C]`. Defaults to `name`.
    #[serde(default)]
    pub label: Option<String>,
    /// Factor applied to the stored value (e.g. 100 when the UI shows %).
    #[serde(default = "one")]
    pub scale: f64,
}

fn one() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub machine_identification_unique: LegacyMachineIdentificationUnique,
    pub measurements: Vec<Column>,
    #[serde(default)]
    pub targets: Vec<Column>,
    /// Range start, Unix ms (inclusive).
    pub from: i64,
    /// Range end, Unix ms (exclusive).
    pub to: i64,
    pub format: Format,
    /// Bucket width in ms; absent or 0 exports every raw sample.
    #[serde(default)]
    pub bucket_ms: Option<i64>,
    /// IANA time zone of the `Time` column, e.g. `Europe/Berlin`. Defaults to UTC.
    #[serde(default)]
    pub timezone: Option<String>,
}

pub async fn post(State(state): State<LegacyApiState>, Json(body): Json<Request>) -> AxumResponse {
    if body.to <= body.from {
        return ResponseUtil::error("`to` must be greater than `from`");
    }
    if body.measurements.is_empty() {
        return ResponseUtil::error("no measurements requested");
    }
    if body.measurements.len() + body.targets.len() > MAX_NAMES {
        return ResponseUtil::error("too many names requested");
    }
    if body
        .measurements
        .iter()
        .chain(&body.targets)
        .any(|c| !c.scale.is_finite())
    {
        return ResponseUtil::error("`scale` must be finite");
    }

    let identity = MachineInstanceIdentification::from(body.machine_identification_unique).to_u64();

    let target_names: Vec<String> = body.targets.iter().map(|c| c.name.clone()).collect();
    let steps = match fetch_targets(&state.db, identity, &target_names, body.from, body.to).await {
        Ok(steps) => steps,
        Err(e) => {
            tracing::error!(error = %e, "history export: target query failed");
            return ResponseUtil::error(&format!("history export failed: {e}"));
        }
    };

    let bucket_ms = body
        .bucket_ms
        .filter(|b| *b > 0)
        .map(|b| b.max(MIN_BUCKET_MS));
    let labels = column_labels(&body, bucket_ms);
    let measurement_names: Vec<String> = body.measurements.iter().map(|c| c.name.clone()).collect();

    // The SQL is assembled from fixed fragments; every value and column name is bound.
    let mut sql =
        String::from("SELECT toTimeZone(t_key, ?) AS ?, toUnixTimestamp64Milli(t_key) AS ?");
    for _ in &body.measurements {
        if bucket_ms.is_some() {
            sql.push_str(
                ", avgOrNullIf(v, name = ?) * ? AS ?\
                 , minOrNullIf(v, name = ?) * ? AS ?\
                 , maxOrNullIf(v, name = ?) * ? AS ?\
                 , countIf(name = ?) AS ?",
            );
        } else {
            sql.push_str(", avgOrNullIf(v, name = ?) * ? AS ?");
        }
    }
    for target in &body.targets {
        let has_steps = steps.get(&target.name).is_some_and(|s| !s.t.is_empty());
        if has_steps {
            sql.push_str(
                ", if(arrayCount(s -> s <= toUnixTimestamp64Milli(t_key), ?) = 0, NULL, \
                 toFloat64(arrayElement(?, arrayCount(s -> s <= toUnixTimestamp64Milli(t_key), ?))) * ?) AS ?",
            );
        } else {
            sql.push_str(", CAST(NULL, 'Nullable(Float64)') AS ?");
        }
    }
    sql.push_str(
        " FROM (
            SELECT timestamp, name, assumeNotNull(value) AS v
            FROM machine_measurements
            WHERE identity = ?
              AND name IN ?
              AND timestamp >= fromUnixTimestamp64Milli(?)
              AND timestamp <  fromUnixTimestamp64Milli(?)
              AND value IS NOT NULL
              AND isFinite(assumeNotNull(value))
        )",
    );
    sql.push_str(if bucket_ms.is_some() {
        " GROUP BY toStartOfInterval(timestamp, toIntervalMillisecond(?)) AS t_key ORDER BY t_key"
    } else {
        " GROUP BY timestamp AS t_key ORDER BY t_key"
    });

    let mut query = state
        .db
        .query(&sql)
        // Empty CSV cells instead of `\N`, which spreadsheets show literally.
        .with_setting("format_csv_null_representation", "")
        .bind(body.timezone.as_deref().unwrap_or("UTC"))
        .bind(Identifier(&labels.time))
        .bind(Identifier(&labels.unix_ms));
    let mut label = labels.columns.iter();
    for column in &body.measurements {
        let stats = if bucket_ms.is_some() { 3 } else { 1 };
        for _ in 0..stats {
            query = query
                .bind(&column.name)
                .bind(column.scale)
                .bind(Identifier(label.next().expect("one label per column")));
        }
        if bucket_ms.is_some() {
            query = query
                .bind(&column.name)
                .bind(Identifier(label.next().expect("one label per column")));
        }
    }
    for target in &body.targets {
        match steps.get(&target.name).filter(|s| !s.t.is_empty()) {
            Some(series) => {
                query = query
                    .bind(&series.t)
                    .bind(&series.v)
                    .bind(&series.t)
                    .bind(target.scale)
                    .bind(Identifier(label.next().expect("one label per column")));
            }
            None => {
                query = query.bind(Identifier(label.next().expect("one label per column")));
            }
        }
    }
    query = query
        .bind(identity)
        .bind(&measurement_names)
        .bind(body.from)
        .bind(body.to);
    if let Some(bucket_ms) = bucket_ms {
        query = query.bind(bucket_ms);
    }

    let mut cursor = match query.fetch_bytes(body.format.clickhouse_format()) {
        Ok(cursor) => cursor,
        Err(e) => return export_error(e),
    };
    // Read the first chunk before answering, so a bad query (unknown time zone, …) is
    // still reported as an error instead of a truncated file.
    let first = match cursor.next().await {
        Ok(first) => first,
        Err(e) => return export_error(e),
    };

    let stream =
        futures_util::stream::unfold((Some(cursor), first), |(cursor, pending)| async move {
            if let Some(chunk) = pending {
                return Some((Ok(chunk), (cursor, None)));
            }
            let mut cursor = cursor?;
            match cursor.next().await {
                Ok(Some(chunk)) => Some((Ok(chunk), (Some(cursor), None))),
                Ok(None) => None,
                Err(e) => {
                    tracing::error!(error = %e, "history export: stream failed");
                    Some((Err(std::io::Error::other(e.to_string())), (None, None)))
                }
            }
        });

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, body.format.content_type())
        .body(Body::from_stream(stream))
        .unwrap()
}

fn export_error(e: clickhouse::error::Error) -> AxumResponse {
    tracing::error!(error = %e, "history export query failed");
    ResponseUtil::error(&format!("history export failed: {e}"))
}

struct Labels {
    time: String,
    unix_ms: String,
    /// In SQL order: per measurement one (raw) or three (mean, min, max) columns, then one per target.
    columns: Vec<String>,
}

/// Column headers, unique (ClickHouse rejects one alias for two expressions). With
/// buckets, the headers name the interval, so nobody mistakes a mean for a sample.
fn column_labels(body: &Request, bucket_ms: Option<i64>) -> Labels {
    let mut taken = HashSet::new();
    let mut unique = |wanted: String| -> String {
        let mut label = wanted.clone();
        let mut n = 2;
        while !taken.insert(label.clone()) {
            label = format!("{wanted} ({n})");
            n += 1;
        }
        label
    };

    let interval = bucket_ms.map(format_interval);
    let time = unique(match &interval {
        Some(_) => "Time (interval start)".to_string(),
        None => "Time".to_string(),
    });
    let unix_ms = unique("Unix ms".to_string());
    let mut columns = Vec::new();
    for column in &body.measurements {
        let base = column.label.clone().unwrap_or_else(|| column.name.clone());
        match &interval {
            Some(interval) => {
                columns.push(unique(format!("{base} (mean over {interval})")));
                columns.push(unique(format!("{base} (min over {interval})")));
                columns.push(unique(format!("{base} (max over {interval})")));
                columns.push(unique(format!("{base} (samples)")));
            }
            None => columns.push(unique(base)),
        }
    }
    for target in &body.targets {
        let base = target.label.clone().unwrap_or_else(|| target.name.clone());
        columns.push(unique(base));
    }
    Labels {
        time,
        unix_ms,
        columns,
    }
}

/// `1 s`, `10 s`, `1 min`, `250 ms`.
fn format_interval(ms: i64) -> String {
    if ms % 3_600_000 == 0 {
        format!("{} h", ms / 3_600_000)
    } else if ms % 60_000 == 0 {
        format!("{} min", ms / 60_000)
    } else if ms % 1_000 == 0 {
        format!("{} s", ms / 1_000)
    } else {
        format!("{ms} ms")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn column(name: &str, label: Option<&str>) -> Column {
        Column {
            name: name.to_string(),
            label: label.map(str::to_string),
            scale: 1.0,
        }
    }

    fn request(measurements: Vec<Column>, targets: Vec<Column>) -> Request {
        Request {
            machine_identification_unique: serde_json::from_str(
                r#"{"machine_identification":{"vendor":1,"machine":4},"serial":1}"#,
            )
            .unwrap(),
            measurements,
            targets,
            from: 0,
            to: 1,
            format: Format::Csv,
            bucket_ms: None,
            timezone: None,
        }
    }

    #[test]
    fn labels_are_unique() {
        let body = request(
            vec![
                column("a", Some("Temp")),
                column("b", Some("Temp")),
                column("c", Some("Time")),
            ],
            vec![column("t", None)],
        );
        let labels = column_labels(&body, None);
        assert_eq!(labels.time, "Time");
        assert_eq!(labels.columns, vec!["Temp", "Temp (2)", "Time (2)", "t"]);
    }

    #[test]
    fn bucketed_labels_have_stats() {
        let body = request(vec![column("a", Some("Temp"))], vec![]);
        let labels = column_labels(&body, Some(60_000));
        assert_eq!(labels.time, "Time (interval start)");
        assert_eq!(
            labels.columns,
            vec![
                "Temp (mean over 1 min)",
                "Temp (min over 1 min)",
                "Temp (max over 1 min)",
                "Temp (samples)"
            ]
        );
    }
}
