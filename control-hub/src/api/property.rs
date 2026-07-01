use std::sync::Arc;

use axum::{Json, extract::{Path, Query, State}};
use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use machine_core::{MachineIdentificationUnique, PropertySpec};
use serde::{Deserialize, Serialize};

use crate::{SharedState, api::types::{Aggregation, Interval, Ordering}};

pub async fn handle(
    State(state): State<Arc<SharedState>>,
    Path((machine_name, serial, property_name)): Path<(String, u32, String)>,
    Query(q): Query<PropertyQuery>,
) -> Result<Json<Samples>, String> {
    let Some(ident) = state.machine_names.get(&machine_name) else {
        return Err(format!("No such machine: {machine_name}"));
    };

    let spec = state.machine_specs.get(ident)
        .expect("if machines_names contains entry so must this!");

    let Some(property_spec) = spec.find_property(&property_name) else {
        return Err(format!("No such property: {property_name}"));
    };

    let table: &str = match property_spec {
        PropertySpec::Integer { .. } | PropertySpec::Boolean => "integer",
        PropertySpec::Float { .. } | PropertySpec::UoM { .. } => "float",
    };

    let uid = MachineIdentificationUnique {
        ident: *ident,
        serial,
    }.as_u64();

    let (from, to) = get_range(&q, Utc::now())?;

    match (q.aggregate, q.interval) {
        (Some(_), None) => {
            Err("'aggregate' requires 'interval'".into())
        }
        (None, Some(_)) => {
            Err("'interval' requires 'aggregate'".into())
        }
        (Some(aggregation), Some(interval)) => {
            let samples = handle_aggregate(
                &state.client,
                uid,
                &property_name,
                table == "integer",
                from,
                to,
                interval, 
                aggregation,
                q.ordering,
                q.limit.unwrap_or(1000).min(100_000),
            ).await?;

            Ok(Json(samples))
        },
        _ => {
            let samples = handle_simple(
                &state.client,
                uid,
                &property_name,
                table == "integer",
                from,
                to,
                q.ordering,
                q.limit.unwrap_or(1000).min(100_000),
            ).await?;

            Ok(Json(samples))
        },
    }
}

pub async fn handle_simple(
    client: &Client,
    ident: u64,
    name: &str,
    is_int: bool,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    ordering: Option<Ordering>,
    limit: u64,
) -> Result<Samples, String> {
    let order = match ordering.unwrap_or(Ordering::Descending) {
        Ordering::Ascending => "ASC",
        Ordering::Descending => "DESC",
    };

    let mut sql = r#"
        SELECT ts, value
        FROM properties_float
        WHERE ident = ?
          AND name = ?
    "#
    .to_string();

    // time filters
    if from.is_some() {
        sql.push_str(" AND ts >= toDateTime64(?, 3)");
    }

    if to.is_some() {
        sql.push_str(" AND ts <= toDateTime64(?, 3)");
    }

    sql.push_str(" ORDER BY ts ");
    sql.push_str(order);
    sql.push_str(" LIMIT ?");

    println!("query: {sql}");

    let mut query = client
        .query(&sql)
        .bind(ident)
        .bind(name);

    if let Some(from) = from {
        let ts = to_clickhouse_datetime64_string(from);
        query = query.bind(ts);
    }

    if let Some(to) = to {
        let ts = to_clickhouse_datetime64_string(to);
        query = query.bind(ts);
    }

    query = query.bind(limit);

    // execute with correct type
    if is_int {
        let rows = query
            .fetch_all::<Sample<i64>>()
            .await
            .map_err(|e| e.to_string())?;

        Ok(Samples::Integer(rows))
    } else {
        let rows = query
            .fetch_all::<Sample<f64>>()
            .await
            .map_err(|e| e.to_string())?;

        Ok(Samples::Float(rows))
    }
}

pub async fn handle_aggregate(
    client: &Client,
    ident: u64,
    name: &str,
    is_int: bool,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    interval: Interval,
    aggregation: Aggregation,
    ordering: Option<Ordering>,
    limit: u64,
) -> Result<Samples, String> {
    let aggregate = match aggregation {
        Aggregation::Average => "avg(value)",
        Aggregation::Median => "median(value)",
        Aggregation::Min => "min(value)",
        Aggregation::Max => "max(value)",
        Aggregation::Sum => "sum(value)",
        Aggregation::Count => "count()",
        Aggregation::First => "argMin(value, ts)",
        Aggregation::Last => "argMax(value, ts)",
    };

    let interval_sql = interval.to_clickhouse_interval();

    let ordering = ordering.map(|v| match v {
        Ordering::Ascending => "ASC",
        Ordering::Descending => "DESC",
    });

    let mut sql = format!(
        r#"
        SELECT
            toDateTime64(toStartOfInterval(ts, {interval_sql}), 3) AS ts,
            {aggregate} AS value
        FROM properties_float
        WHERE ident = ?
          AND name = ?
        "#
    );

    // time filters
    if from.is_some() {
        sql.push_str(" AND ts >= toDateTime64(?, 3)");
    }

    if to.is_some() {
        sql.push_str(" AND ts <= toDateTime64(?, 3)");
    }

    sql.push_str(
        r#"
        GROUP BY ts
        ORDER BY ts "#,
    );

    if let Some(v) = ordering {
        sql.push_str(v);
    }

    sql.push_str(" LIMIT ?");

    let mut query = client.query(&sql).bind(ident).bind(name);

    if let Some(from) = from {
        let ts = to_clickhouse_datetime64_string(from);
        query = query.bind(ts);
    }

    if let Some(to) = to {
        let ts = to_clickhouse_datetime64_string(to);
        query = query.bind(ts);
    }

    query = query.bind(limit);

    if is_int {
        let samples = query
            .fetch_all::<Sample<i64>>()
            .await
            .map_err(|e| e.to_string())?;

        Ok(Samples::Integer(samples))
    } else {
        let samples = query
            .fetch_all::<Sample<f64>>()
            .await
            .map_err(|e| e.to_string())?;

        Ok(Samples::Float(samples))
    }
}
#[derive(Deserialize)]
pub struct PropertyQuery {
    /// Start timestamp.
    #[serde(default, with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub from: Option<DateTime<Utc>>,

    /// End timestamp.
    #[serde(default, with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub to: Option<DateTime<Utc>>,

    /// Read the last x duration (s, m, d, h supported).
    pub last: Option<Interval>,

    /// Interval for aggregation
    pub interval: Option<Interval>,

    /// Aggregation method
    pub aggregate: Option<Aggregation>,

    /// Maximum number of samples to return.
    pub limit: Option<u64>,

    pub ordering: Option<Ordering>,

    // /// Response format.
    // pub format: Option<ResponseFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Samples {
    Float(Vec<Sample<f64>>),
    Integer(Vec<Sample<i64>>),
    Boolean(Vec<Sample<bool>>),
    String(Vec<Sample<String>>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Row)]
pub struct Sample<T> {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    ts: DateTime<Utc>,
    value: T
}

fn get_range(
    q: &PropertyQuery,
    now: DateTime<Utc>,
) -> Result<(Option<DateTime<Utc>>, Option<DateTime<Utc>>), String> {
    match (q.last.clone(), q.from, q.to) {
        // last only
        (Some(last), None, None) => {
            let duration = last.to_duration();
            Ok((Some(now - duration), None))
        }
        // from and to
        (None, Some(from), Some(to)) => Ok((Some(from), Some(to))),
        // from only
        (None, Some(from), None) => Ok((Some(from), None)),
        // to only
        (None, None, Some(to)) => Ok((None, Some(to))) ,
        // nothing
        (None, None, None) => Ok((None, None)),
        // invalid combinations
        (Some(_), _, _) => {
            Err("'last' cannot be combined with 'from' or 'to'".into())
        }
    }
}

fn to_clickhouse_datetime64_string(dt: DateTime<Utc>) -> String {
    let secs = dt.timestamp();
    let nanos = dt.timestamp_subsec_nanos();

    // convert nanos → fractional seconds (up to 9 digits)
    let frac = nanos as f64 / 1_000_000_000.0;

    let value = secs as f64 + frac;

    // format with 3 decimals for DateTime64(3)
    format!("{:.3}", value)
}