//! `POST /api/v1/machine/history`: downsampled measurement history and step-wise
//! config (target) history from ClickHouse for the graph pages.
//!
//! Measurements are reduced inside ClickHouse to the min and max of each time bucket
//! (with their timestamps), so a spike is never lost and the response stays at about
//! two points per bucket no matter how many raw rows the range holds. Buckets are
//! aligned to the Unix epoch, so the UI can fetch only the newest buckets of a live
//! view and splice them onto what it already has.

use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use axum::response::Response as AxumResponse;
use qitech_framework::MachineInstanceIdentification;
use serde::Deserialize;
use serde::Serialize;

use crate::api::legacy::LegacyApiState;
use crate::api::legacy::response_util::ResponseUtil;
use crate::api::legacy::types::LegacyMachineIdentificationUnique;

/// Upper bound on buckets per series, whatever the client asks for.
const MAX_BUCKETS: i64 = 20_000;
/// Upper bound on names per request.
const MAX_NAMES: usize = 64;
/// Older config records fetched to find the value in effect at `from`.
const TARGET_LOOKBACK_ROWS: u64 = 64;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub machine_identification_unique: LegacyMachineIdentificationUnique,
    /// Measurement paths, e.g. `heating.nozzle.temperature`.
    #[serde(default)]
    pub measurements: Vec<String>,
    /// Config property paths whose accepted values form a step series, e.g. `screw.target_rpm`.
    #[serde(default)]
    pub targets: Vec<String>,
    /// Range start, Unix ms (inclusive).
    pub from: i64,
    /// Range end, Unix ms (exclusive).
    pub to: i64,
    /// Requested bucket width in ms. Raised server-side if the range would exceed `MAX_BUCKETS`.
    pub bucket_ms: i64,
}

/// Columnar series: `t[i]` (Unix ms) belongs to `v[i]`.
#[derive(Debug, Default, Serialize)]
pub struct Series {
    pub t: Vec<i64>,
    pub v: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    /// Bucket width actually used.
    pub bucket_ms: i64,
    pub measurements: HashMap<String, Series>,
    pub targets: HashMap<String, Series>,
}

#[derive(Debug, Deserialize, clickhouse::Row)]
struct BucketRow {
    /// 1-based index into the requested measurement names.
    idx: u64,
    t_min: i64,
    v_min: f64,
    t_max: i64,
    v_max: f64,
}

#[derive(Debug, Deserialize, clickhouse::Row)]
struct ConfigRow {
    name: String,
    t: i64,
    data: String,
}

pub async fn post(State(state): State<LegacyApiState>, Json(body): Json<Request>) -> AxumResponse {
    if body.to <= body.from {
        return ResponseUtil::error("`to` must be greater than `from`");
    }
    if body.measurements.len() + body.targets.len() > MAX_NAMES {
        return ResponseUtil::error("too many names requested");
    }

    let identity = MachineInstanceIdentification::from(body.machine_identification_unique).to_u64();
    let span = body.to - body.from;
    let bucket_ms = body
        .bucket_ms
        .max(1)
        .max((span + MAX_BUCKETS - 1) / MAX_BUCKETS);

    let (measurements, targets) = tokio::join!(
        fetch_measurements(&state.db, identity, &body, bucket_ms),
        fetch_targets(&state.db, identity, &body),
    );

    match (measurements, targets) {
        (Ok(measurements), Ok(targets)) => ResponseUtil::ok(Response {
            bucket_ms,
            measurements,
            targets,
        }),
        (Err(e), _) | (_, Err(e)) => {
            tracing::error!(error = %e, "history query failed");
            ResponseUtil::error(&format!("history query failed: {e}"))
        }
    }
}

async fn fetch_measurements(
    db: &clickhouse::Client,
    identity: u64,
    body: &Request,
    bucket_ms: i64,
) -> Result<HashMap<String, Series>, clickhouse::error::Error> {
    if body.measurements.is_empty() {
        return Ok(HashMap::new());
    }

    // Reads only the (identity, name, timestamp) primary-key range, one pass, parallel GROUP BY.
    let rows = db
        .query(
            "SELECT
                indexOf(?, name)                                AS idx,
                argMin(toUnixTimestamp64Milli(timestamp), v)    AS t_min,
                min(v)                                          AS v_min,
                argMax(toUnixTimestamp64Milli(timestamp), v)    AS t_max,
                max(v)                                          AS v_max
            FROM
            (
                SELECT timestamp, name, assumeNotNull(value) AS v
                FROM machine_measurements
                WHERE identity = ?
                  AND name IN ?
                  AND timestamp >= fromUnixTimestamp64Milli(?)
                  AND timestamp <  fromUnixTimestamp64Milli(?)
                  AND value IS NOT NULL
                  AND isFinite(assumeNotNull(value))
            )
            GROUP BY idx, intDiv(toUnixTimestamp64Milli(timestamp), ?) AS bucket
            ORDER BY idx, bucket",
        )
        .bind(&body.measurements)
        .bind(identity)
        .bind(&body.measurements)
        .bind(body.from)
        .bind(body.to)
        .bind(bucket_ms)
        .fetch_all::<BucketRow>()
        .await?;

    let mut out: HashMap<String, Series> = body
        .measurements
        .iter()
        .map(|name| (name.clone(), Series::default()))
        .collect();

    for row in rows {
        let Some(name) = body.measurements.get((row.idx as usize).wrapping_sub(1)) else {
            continue;
        };
        let series = out.get_mut(name).expect("initialised above");

        // Emit min and max in time order; collapse when they are the same sample.
        if row.t_min == row.t_max {
            series.t.push(row.t_min);
            series.v.push(row.v_min);
        } else if row.t_min < row.t_max {
            series.t.extend([row.t_min, row.t_max]);
            series.v.extend([row.v_min, row.v_max]);
        } else {
            series.t.extend([row.t_max, row.t_min]);
            series.v.extend([row.v_max, row.v_min]);
        }
    }

    Ok(out)
}

/// Step series of accepted values per config property. The first point is the value in
/// effect at `from` (if any record precedes the range), placed at `from`.
async fn fetch_targets(
    db: &clickhouse::Client,
    identity: u64,
    body: &Request,
) -> Result<HashMap<String, Series>, clickhouse::error::Error> {
    if body.targets.is_empty() {
        return Ok(HashMap::new());
    }

    // Config writes are rare, so reading every record in range is cheap. The lookback
    // query fetches the most recent records before the range per name, newest first.
    let (in_range, before) = tokio::join!(
        db.query(
            "SELECT CAST(name, 'String') AS name,
                    toUnixTimestamp64Milli(timestamp) AS t,
                    toJSONString(event_data) AS data
             FROM machine_config_property_records
             WHERE identity = ?
               AND name IN ?
               AND event_type IN ('Registered', 'Written')
               AND timestamp >= fromUnixTimestamp64Milli(?)
               AND timestamp <  fromUnixTimestamp64Milli(?)
             ORDER BY timestamp",
        )
        .bind(identity)
        .bind(&body.targets)
        .bind(body.from)
        .bind(body.to)
        .fetch_all::<ConfigRow>(),
        db.query(
            "SELECT CAST(name, 'String') AS name,
                    toUnixTimestamp64Milli(timestamp) AS t,
                    toJSONString(event_data) AS data
             FROM machine_config_property_records
             WHERE identity = ?
               AND name IN ?
               AND event_type IN ('Registered', 'Written')
               AND timestamp < fromUnixTimestamp64Milli(?)
             ORDER BY timestamp DESC
             LIMIT ? BY name",
        )
        .bind(identity)
        .bind(&body.targets)
        .bind(body.from)
        .bind(TARGET_LOOKBACK_ROWS)
        .fetch_all::<ConfigRow>(),
    );
    let (in_range, before) = (in_range?, before?);

    let mut out: HashMap<String, Series> = body
        .targets
        .iter()
        .map(|name| (name.clone(), Series::default()))
        .collect();

    // Value in effect at `from`: the newest accepted record before the range.
    for row in &before {
        let Some(series) = out.get_mut(&row.name) else {
            continue;
        };
        if !series.t.is_empty() {
            continue;
        }
        if let Some(value) = accepted_value(&row.data) {
            series.t.push(body.from);
            series.v.push(value);
        }
    }

    for row in &in_range {
        let Some(series) = out.get_mut(&row.name) else {
            continue;
        };
        let Some(value) = accepted_value(&row.data) else {
            continue;
        };
        // Skip records that do not change the value.
        if series.v.last() == Some(&value) {
            continue;
        }
        series.t.push(row.t);
        series.v.push(value);
    }

    Ok(out)
}

/// Numeric value of a `Registered` (its default) or accepted `Written` config event.
fn accepted_value(data: &str) -> Option<f64> {
    let event: serde_json::Value = serde_json::from_str(data).ok()?;

    let scalar = if let Some(registered) = event.get("Registered") {
        registered.get("default")?
    } else if let Some(written) = event.get("Written") {
        written.get("outcome")?.get("Accepted")?;
        written.get("value")?
    } else {
        return None;
    };

    scalar_to_f64(scalar)
}

fn scalar_to_f64(scalar: &serde_json::Value) -> Option<f64> {
    if let Some(v) = scalar.get("float") {
        return v.as_f64();
    }
    if let Some(v) = scalar.get("integer") {
        return v.as_f64();
    }
    if let Some(v) = scalar.get("boolean") {
        return v.as_bool().map(|b| if b { 1.0 } else { 0.0 });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_value_reads_registered_default() {
        let data = r#"{"Registered":{"default":{"float":12.5},"capability":"ReadWrite","constraints":{}}}"#;
        assert_eq!(accepted_value(data), Some(12.5));
    }

    #[test]
    fn accepted_value_reads_accepted_write() {
        let data = r#"{"Written":{"value":{"float":200.0},"origin":"Api","outcome":{"Accepted":{"changed":true}}}}"#;
        assert_eq!(accepted_value(data), Some(200.0));
    }

    #[test]
    fn accepted_value_ignores_rejected_write() {
        let data = r#"{"Written":{"value":{"float":999.0},"origin":"Api","outcome":{"Rejected":{"OutOfRange":null}}}}"#;
        assert_eq!(accepted_value(data), None);
    }

    #[test]
    fn accepted_value_reads_integer_and_bool() {
        assert_eq!(
            accepted_value(r#"{"Registered":{"default":{"integer":3}}}"#),
            Some(3.0)
        );
        assert_eq!(
            accepted_value(r#"{"Registered":{"default":{"boolean":true}}}"#),
            Some(1.0)
        );
    }
}
