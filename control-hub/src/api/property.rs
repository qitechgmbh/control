use std::sync::Arc;

use axum::{Json, extract::{Path, Query, State}};
use chrono::{DateTime, Utc};
use machine_core::{MachineIdentification, MachineIdentificationUnique, MachineSpec};
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

#[derive(Debug, Clone, Serialize)]
pub enum Samples {
    Float(Vec<Sample<f64>>),
    Integer(Vec<Sample<i64>>),
    Boolean(Vec<Sample<bool>>),
    String(Vec<Sample<String>>),
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Sample<T> {
    ts: u64,
    value: T
}

pub async fn handler(
    State(state): State<Arc<SharedState>>,
    Path((machine_name, serial, property_name)): Path<(String, u32, String)>,
    Query(query): Query<PropertyQuery>,
) -> Result<Json<Samples>, String> {
    let Some(spec) = find_spec_by_name(&state, &machine_name) else {
        return Err(format!("No such machine: {machine_name}"));
    };

    let Some(property_spec) = spec.find_property(&property_name) else {
        return Err(format!("No such property: {machine_name}"));
    };

   // spec.properties.get(k)

    // spec.properties.get(k);

    let uid = MachineIdentificationUnique {
        ident,
        serial,
    }.as_u64();

    let mut sql = String::from(
        r#"
        SELECT ts, value
        FROM properties_float
        WHERE ident = ?
        AND name = ?
        "#,
    );

    if from.is_some() {
        sql.push_str(" AND ts >= ?");
    }

    if to.is_some() {
        sql.push_str(" AND ts <= ?");
    }

    sql.push_str(" ORDER BY ts DESC LIMIT ?");

    let mut query = state
        .client
        .query(&sql)
        .bind(uid)
        .bind(property_name);

    if let Some(from) = from {
        query = query.bind(from);
    }

    if let Some(to) = to {
        query = query.bind(to);
    }

    query = query.bind(limit);

    let samples = query
        .fetch_all::<Sample<f64>>()
        .await
        .map_err(|e| e.to_string())?;

    print!("samples: {ident:?}\n");

    Ok(Json(samples))
}

fn find_spec_by_name(state: &SharedState, name: &str) -> Option<&MachineSpec> {
    for (ident, spec) in state.machine_specs.iter() {
        if spec.name == name {
            return Some(spec);
        }
    }

    return None;
}

fn deconstruct_property_name(name: String) -> Vec<String> {
    name.split('.')
        .map(|s| s.to_string())
        .collect()
}