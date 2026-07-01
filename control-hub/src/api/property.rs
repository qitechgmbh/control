use std::sync::Arc;

use axum::{Json, extract::{Path, Query, State}};
use chrono::{DateTime, Utc};
use clickhouse::Row;
use machine_core::{MachineIdentificationUnique, PropertySpec};
use serde::{Deserialize, Serialize};

use crate::SharedState;

#[derive(Debug, Deserialize)]
pub struct PropertyQuery {
    /// Start timestamp (e.g. unix ns/ms/s depending on your API).
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub from: Option<DateTime<Utc>>,

    /// End timestamp.
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub to: Option<DateTime<Utc>>,

    /// Read the last x duration (s, m, d, h supported).
    pub last: SimpleDateTime,

    /// Interval for aggregation
    pub interval: SimpleDateTime,

    /// Aggregation method
    pub aggregate: Aggregation,

    /// Maximum number of samples to return.
    pub limit: Option<u64>,

    /// Response format.
    pub format: Option<ResponseFormat>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    Json,
    Binary,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SimpleDateTime {
    Seconds(f64),
    Minutes(f64),
    Hours(f64),
    Days(f64),
}

impl std::str::FromStr for SimpleDateTime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // find first letter (splits number + unit)
        let split = s
            .chars()
            .position(|c| c.is_alphabetic())
            .ok_or("missing unit")?;

        let (num_str, unit) = s.split_at(split);

        let value: f64 = num_str
            .parse()
            .map_err(|_| "invalid number")?;

        match unit {
            "s" => Ok(Self::Seconds(value)),
            "m" => Ok(Self::Minutes(value)),
            "h" => Ok(Self::Hours(value)),
            "d" => Ok(Self::Days(value)),
            _ => Err(format!("invalid unit: {}", unit)),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Aggregation {
    Min,
    Max,
    Avg,
    Median,
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
    ts: u64,
    value: T
}

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
        return Err(format!("No such property: {machine_name}"));
    };

    let table: &str = match property_spec {
        PropertySpec::Integer { .. } | PropertySpec::Boolean => "integer",
        PropertySpec::Float { .. } | PropertySpec::UoM { .. } => "float",
    };

    let uid = MachineIdentificationUnique {
        ident: *ident,
        serial,
    }.as_u64();

    let mut sql = format!(r#"
        SELECT ts, value
        FROM properties_{table}
        WHERE ident = ?
        AND name = ?
    "#,);

    if q.from.is_some() {
        sql.push_str(" AND ts >= ?");
    }

    if q.to.is_some() {
        sql.push_str(" AND ts <= ?");
    }

    sql.push_str(" ORDER BY ts DESC LIMIT ?");

    let mut query = state
        .client
        .query(&sql)
        .bind(uid)
        .bind(property_name);

    if let Some(from) = q.from {
        query = query.bind(from);
    }

    if let Some(to) = q.to {
        query = query.bind(to);
    }

    query = query.bind(q.limit.unwrap_or(1000).min(1000));

    match property_spec {
        PropertySpec::Boolean | PropertySpec::Integer { .. } => {
            let samples = query
                .fetch_all::<Sample<i64>>()
                .await
                .map_err(|e| e.to_string())?;

            Ok(Json(Samples::Integer(samples)))
        },
        PropertySpec::Float { .. } | PropertySpec::UoM  { .. } => {
            let samples = query
                .fetch_all::<Sample<f64>>()
                .await
                .map_err(|e| e.to_string())?;

            Ok(Json(Samples::Float(samples)))
        },
    }
}
