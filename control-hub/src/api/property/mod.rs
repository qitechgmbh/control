use std::sync::Arc;

use axum::{Json, extract::{Path, Query, State}};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use clickhouse::Row;

use crate::{SharedState, api::{types::Table, utils}};

const LIMIT_DEFAULT: u64 = 100;
const LIMIT_MAXIMUM: u64 = 1_000_000;

mod query;
use query::PropertyQuery;

mod operation;
use operation::Operation;

pub async fn handle(
    State(state): State<Arc<SharedState>>,
    Path((slug, serial, property)): Path<(String, u32, String)>,
    Query(q): Query<PropertyQuery>,
) -> Result<Json<Samples>, String> {
    let (ident, spec) = utils::machine_info_from_slug(&state, &slug)?;

    let Some(property_spec) = spec.find_property(&property) else {
        return Err(format!("No such property: {property}"));
    };

    let op = Operation {
        table: Table::from_property_spec(property_spec),
        machine_uid: utils::init_uid(ident, serial),
        property,
        time_span: q.get_time_span(Utc::now())?,
        aggregation: q.get_aggregation()?,
        ordering: q.get_ordering(),
        limit: q.get_limit(),
    };

    let samples = op.execute(&state.client).await?;
    Ok(Json(samples))
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
