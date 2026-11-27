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
use clickhouse::sql::Identifier;
use serde::Deserialize;

use super::history::fetch_targets;
use super::query::ApiError;
use super::query::ApiResult;
use super::query::RAW_SAMPLES;
use super::query::TimeRange;
use super::query::check_name_count;
use crate::api::legacy::LegacyApiState;
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
    #[serde(flatten)]
    pub range: TimeRange,
    pub format: Format,
    /// Bucket width in ms; absent or 0 exports every raw sample.
    #[serde(default)]
    pub bucket_ms: Option<i64>,
    /// IANA time zone of the `Time` column, e.g. `Europe/Berlin`. Defaults to UTC.
    #[serde(default)]
    pub timezone: Option<String>,
}

pub async fn post(State(state): State<LegacyApiState>, Json(body): Json<Request>) -> ApiResult {
    body.range.validate()?;
    if body.measurements.is_empty() {
        return Err(ApiError::BadRequest("no measurements requested"));
    }
    check_name_count(body.measurements.len() + body.targets.len())?;
    if body
        .measurements
        .iter()
        .chain(&body.targets)
        .any(|c| !c.scale.is_finite())
    {
        return Err(ApiError::BadRequest("`scale` must be finite"));
    }

    let identity = body.machine_identification_unique.identity();

    let target_names: Vec<String> = body.targets.iter().map(|c| c.name.clone()).collect();
    let steps = fetch_targets(&state.db, identity, &target_names, body.range)
        .await
        .map_err(ApiError::db("history export target query"))?;

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
    // Per target its step series, or `None` without any accepted value (an empty column).
    let target_steps: Vec<_> = body
        .targets
        .iter()
        .map(|target| steps.get(&target.name).filter(|s| !s.t.is_empty()))
        .collect();
    // A step column holds the value in effect at the row time: the number of steps at
    // or before it indexes the values. The row time is the row's (bucket start) time but
    // not before `from`, where the first step (the value in effect at `from`) sits.
    for (i, series) in target_steps.iter().enumerate() {
        if series.is_some() {
            sql.push_str(&format!(
                ", if((arrayCount(s -> s <= greatest(toUnixTimestamp64Milli(t_key), ?), ?) AS step_{i}) = 0, \
                 NULL, toFloat64(arrayElement(?, step_{i})) * ?) AS ?"
            ));
        } else {
            sql.push_str(", CAST(NULL, 'Nullable(Float64)') AS ?");
        }
    }
    sql.push_str(&format!(" FROM ({RAW_SAMPLES})"));
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
    for (target, series) in body.targets.iter().zip(&target_steps) {
        match series {
            Some(series) => {
                query = query
                    .bind(body.range.from)
                    .bind(&series.t)
                    .bind(&series.v)
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
        .bind(body.range.from)
        .bind(body.range.to);
    if let Some(bucket_ms) = bucket_ms {
        query = query.bind(bucket_ms);
    }

    let mut cursor = query
        .fetch_bytes(body.format.clickhouse_format())
        .map_err(ApiError::db("history export"))?;
    // Read the first chunk before answering, so a bad query (unknown time zone, …) is
    // still reported as an error instead of a truncated file.
    let first = cursor
        .next()
        .await
        .map_err(ApiError::db("history export"))?;

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

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, body.format.content_type())
        .body(Body::from_stream(stream))
        .unwrap())
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
            range: TimeRange { from: 0, to: 1 },
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
