//! `POST /api/v1/machine/history`: downsampled measurement history and step-wise
//! config (target) history from ClickHouse for the graph pages.
//!
//! Measurements are reduced inside ClickHouse to one point per time bucket: the mean
//! (drawn as the line) plus the min and max (drawn as a faint band, so a spike is never
//! lost). The response stays at one point per bucket no matter how many raw rows the
//! range holds. Buckets are
//! aligned to the Unix epoch, so the UI can fetch only the newest buckets of a live
//! view and splice them onto what it already has.

use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde::Serialize;

use super::query::ApiError;
use super::query::ApiResult;
use super::query::RAW_SAMPLES;
use super::query::TimeRange;
use super::query::check_name_count;
use crate::api::legacy::LegacyApiState;
use crate::api::legacy::response_util::ResponseUtil;
use crate::api::legacy::types::LegacyMachineIdentificationUnique;

/// Upper bound on buckets per series, whatever the client asks for.
const MAX_BUCKETS: i64 = 20_000;
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
    #[serde(flatten)]
    pub range: TimeRange,
    /// Requested bucket width in ms. Raised server-side if the range would exceed `MAX_BUCKETS`.
    pub bucket_ms: i64,
}

/// Columnar series: `t[i]` (Unix ms) belongs to `v[i]`.
#[derive(Debug, Default, Serialize)]
pub struct Series {
    pub t: Vec<i64>,
    pub v: Vec<f64>,
}

/// One point per bucket: `t[i]` is the mean sample time (Unix ms), `v[i]` the mean,
/// `min[i]`/`max[i]` the extremes and `n[i]` the number of samples of the bucket.
#[derive(Debug, Default, Serialize)]
pub struct BucketSeries {
    pub t: Vec<i64>,
    pub v: Vec<f64>,
    pub min: Vec<f64>,
    pub max: Vec<f64>,
    pub n: Vec<u64>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    /// Bucket width actually used.
    pub bucket_ms: i64,
    pub measurements: HashMap<String, BucketSeries>,
    pub targets: HashMap<String, Series>,
}

#[derive(Debug, Deserialize, clickhouse::Row)]
struct BucketRow {
    /// 1-based index into the requested measurement names.
    idx: u64,
    t: i64,
    v_avg: f64,
    v_min: f64,
    v_max: f64,
    n: u64,
}

#[derive(Debug, Deserialize, clickhouse::Row)]
struct ConfigRow {
    name: String,
    t: i64,
    data: String,
}

pub async fn post(State(state): State<LegacyApiState>, Json(body): Json<Request>) -> ApiResult {
    body.range.validate()?;
    check_name_count(body.measurements.len() + body.targets.len())?;

    let identity = body.machine_identification_unique.identity();
    let bucket_ms = cap_bucket(body.bucket_ms, body.range.span());

    let (measurements, targets) = tokio::try_join!(
        fetch_measurements(&state.db, identity, &body, bucket_ms),
        fetch_targets(&state.db, identity, &body.targets, body.range),
    )
    .map_err(ApiError::db("history query"))?;

    Ok(ResponseUtil::ok(Response {
        bucket_ms,
        measurements,
        targets,
    }))
}

/// Slot widths (ms) of the aggregate projections of `machine_measurements`, coarsest first
/// (see `init_machine_measurements.sql`).
const PROJECTION_SLOTS_MS: [i64; 2] = [60_000, 5_000];

/// The requested bucket width, raised if `span` would need more than `MAX_BUCKETS`. A
/// raised width is rounded up to whole projection slots so it can still use them.
fn cap_bucket(requested_ms: i64, span: i64) -> i64 {
    let min_ms = (span + MAX_BUCKETS - 1) / MAX_BUCKETS;
    if requested_ms >= min_ms {
        return requested_ms.max(1);
    }
    // From 1 s up, a few percent wider buckets cost nothing next to reading raw rows.
    let slot = match min_ms {
        60_000.. => 60_000,
        1_000.. => 5_000,
        _ => return min_ms.max(1),
    };
    (min_ms + slot - 1) / slot * slot
}

/// Slot of the coarsest projection that tiles `bucket_ms` exactly, so every bucket is a
/// whole number of slots; `None` reads the raw rows.
fn projection_slot(bucket_ms: i64) -> Option<i64> {
    PROJECTION_SLOTS_MS
        .into_iter()
        .find(|slot| bucket_ms >= *slot && bucket_ms % slot == 0)
}

/// Bucket query over a projection. Every expression must be spelled exactly as in the
/// projection definition, otherwise ClickHouse silently falls back to the raw rows
/// (`projection_query_matches_migration` guards this).
fn projection_query(slot_ms: i64) -> String {
    let slot = format!("intDiv(toUnixTimestamp64Milli(timestamp), {slot_ms})");
    let ok = "ifNull(isFinite(value), 0)";
    format!(
        "SELECT
            indexOf(?, name)                                                AS idx,
            countIf({ok})                                                   AS n,
            toInt64(sumIf(toUnixTimestamp64Milli(timestamp), {ok}) / n)     AS t,
            sumIf(assumeNotNull(value), {ok}) / n                           AS v_avg,
            minIf(assumeNotNull(value), {ok})                               AS v_min,
            maxIf(assumeNotNull(value), {ok})                               AS v_max
        FROM machine_measurements
        WHERE identity = ?
          AND name IN ?
          AND {slot} >= ?
          AND {slot} < ?
        GROUP BY idx, intDiv({slot}, ?) AS bucket
        HAVING n > 0
        ORDER BY idx, bucket"
    )
}

async fn fetch_measurements(
    db: &clickhouse::Client,
    identity: u64,
    body: &Request,
    bucket_ms: i64,
) -> Result<HashMap<String, BucketSeries>, clickhouse::error::Error> {
    if body.measurements.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = match projection_slot(bucket_ms) {
        // Pre-aggregated slots: a 30 day range reads ~43k rows per series instead of ~83M.
        // The range is widened to whole slots, at most one slot on each side.
        Some(slot_ms) => {
            db.query(&projection_query(slot_ms))
                .bind(&body.measurements)
                .bind(identity)
                .bind(&body.measurements)
                .bind(body.range.from.div_euclid(slot_ms))
                .bind((body.range.to + slot_ms - 1).div_euclid(slot_ms))
                .bind(bucket_ms / slot_ms)
                .fetch_all::<BucketRow>()
                .await?
        }

        // Reads only the (identity, name, timestamp) primary-key range, one pass, parallel GROUP BY.
        None => {
            db.query(&format!(
                "SELECT
                    indexOf(?, name)                                    AS idx,
                    toInt64(avg(toUnixTimestamp64Milli(timestamp)))     AS t,
                    avg(v)                                              AS v_avg,
                    min(v)                                              AS v_min,
                    max(v)                                              AS v_max,
                    count()                                             AS n
                FROM ({RAW_SAMPLES})
                GROUP BY idx, intDiv(toUnixTimestamp64Milli(timestamp), ?) AS bucket
                ORDER BY idx, bucket"
            ))
            .bind(&body.measurements)
            .bind(identity)
            .bind(&body.measurements)
            .bind(body.range.from)
            .bind(body.range.to)
            .bind(bucket_ms)
            .fetch_all::<BucketRow>()
            .await?
        }
    };

    let mut out: HashMap<String, BucketSeries> = body
        .measurements
        .iter()
        .map(|name| (name.clone(), BucketSeries::default()))
        .collect();

    for row in rows {
        let Some(name) = body.measurements.get((row.idx as usize).wrapping_sub(1)) else {
            continue;
        };
        let series = out.get_mut(name).expect("initialised above");
        series.t.push(row.t);
        series.v.push(row.v_avg);
        series.min.push(row.v_min);
        series.max.push(row.v_max);
        series.n.push(row.n);
    }

    Ok(out)
}

/// Step series of accepted values per config property. The first point is the value in
/// effect at `from` (if any record precedes the range), placed at `from`.
pub(super) async fn fetch_targets(
    db: &clickhouse::Client,
    identity: u64,
    targets: &[String],
    range: TimeRange,
) -> Result<HashMap<String, Series>, clickhouse::error::Error> {
    let TimeRange { from, to } = range;
    if targets.is_empty() {
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
        .bind(targets)
        .bind(from)
        .bind(to)
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
        .bind(targets)
        .bind(from)
        .bind(TARGET_LOOKBACK_ROWS)
        .fetch_all::<ConfigRow>(),
    );
    let (in_range, before) = (in_range?, before?);

    let mut out: HashMap<String, Series> = targets
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
            series.t.push(from);
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
    fn picks_the_coarsest_projection_that_tiles_the_bucket() {
        assert_eq!(projection_slot(1_800_000), Some(60_000));
        assert_eq!(projection_slot(60_000), Some(60_000));
        assert_eq!(projection_slot(30_000), Some(5_000));
        assert_eq!(projection_slot(15_000), Some(5_000));
        assert_eq!(projection_slot(5_000), Some(5_000));
        assert_eq!(projection_slot(2_000), None);
        assert_eq!(projection_slot(64_801), None);
    }

    /// Output columns (the `AS` aliases before `FROM`) of a bucket query.
    fn output_columns(query: &str) -> Vec<String> {
        let select = &query[..query.find("FROM").expect("has FROM")];
        let mut columns: Vec<String> = select
            .split(" AS ")
            .skip(1)
            .map(|rest| rest.split(',').next().unwrap().trim().to_owned())
            .collect();
        columns.sort();
        columns
    }

    /// The `clickhouse` crate rejects a result whose columns differ from `BucketRow`'s
    /// fields, failing every request of that path (this broke the long graph ranges once).
    #[test]
    fn projection_query_columns_match_bucket_row() {
        let mut fields = vec!["idx", "n", "t", "v_avg", "v_max", "v_min"];
        fields.sort();
        for slot_ms in PROJECTION_SLOTS_MS {
            assert_eq!(output_columns(&projection_query(slot_ms)), fields);
        }
    }

    #[test]
    fn raised_buckets_stay_on_projection_slots() {
        const DAY: i64 = 86_400_000;
        // requested width kept when it fits
        assert_eq!(cap_bucket(1_800_000, 30 * DAY), 1_800_000);
        // 30 days at 1 ms: raised to 129.6 s, rounded up to 3 whole minutes
        assert_eq!(cap_bucket(1, 30 * DAY), 180_000);
        assert_eq!(projection_slot(cap_bucket(1, 30 * DAY)), Some(60_000));
        // 1 day at 1 ms: 4.32 s, below a minute, rounded to 5 s slots
        assert_eq!(cap_bucket(1, DAY), 5_000);
        // short spans stay raw
        assert_eq!(cap_bucket(1, 60_000), 3);
    }

    /// The query must spell every expression like the projections, or ClickHouse quietly
    /// reads the raw rows again and the long graph ranges get slow.
    #[test]
    fn projection_query_matches_migration() {
        let init = include_str!("../../../../migrations/init_machine_measurements.sql");
        let upgrade =
            include_str!("../../../../migrations/upgrade_machine_measurements_projections.sql");
        for slot_ms in PROJECTION_SLOTS_MS {
            let query = projection_query(slot_ms);
            let slot = format!("intDiv(toUnixTimestamp64Milli(timestamp), {slot_ms})");
            for expr in [
                slot.clone(),
                "countIf(ifNull(isFinite(value), 0))".to_owned(),
                "sumIf(assumeNotNull(value), ifNull(isFinite(value), 0))".to_owned(),
                "minIf(assumeNotNull(value), ifNull(isFinite(value), 0))".to_owned(),
                "maxIf(assumeNotNull(value), ifNull(isFinite(value), 0))".to_owned(),
                "sumIf(toUnixTimestamp64Milli(timestamp), ifNull(isFinite(value), 0))".to_owned(),
            ] {
                assert!(query.contains(&expr), "query lacks {expr}");
                assert!(init.contains(&expr), "init migration lacks {expr}");
                assert!(upgrade.contains(&expr), "upgrade migration lacks {expr}");
            }
        }
    }

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
