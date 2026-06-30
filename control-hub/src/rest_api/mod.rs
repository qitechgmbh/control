use std::{collections::{BTreeMap, HashMap}, fmt::Debug, io, sync::Arc};
use axum::{Json, extract::{Path, Query, State}, routing::get};
use chrono::{DateTime, Utc};
use clickhouse::{Row};
use machine_core::{MachineIdentification, MachineIdentificationUnique, MachineSpec};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

use crate::SharedState;

#[derive(Debug, Serialize, Deserialize, Row)]
struct PropertyRow<T> {
    ts: DateTime<Utc>,
    value: T,
}

#[derive(Debug, Serialize)]
struct PropertyHistoryResponse {
    machine_id: u64,
    property_name: String,
    samples: Samples,
}

#[derive(Debug, Serialize)]
enum Samples {
    Float(Vec<PropertyRow<f64>>),
    Integer(Vec<PropertyRow<i64>>),
    Boolean(Vec<PropertyRow<bool>>),
    String(Vec<PropertyRow<String>>),
}

pub struct Config {
    pub address: String,
}

pub async fn run(state: SharedState, config: Config) -> io::Result<()> {
    let app = axum::Router::new()
        .route("/api/v2/machines", get(machines))
        .route("/api/v2/machines/{name}/{serial}", get(machine))
        .route("/api/v2/machines/{name}/{serial}/{property_name}", get(property))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(&config.address)
        .await
        .unwrap();

    println!("[RestApi] Listening on {}", &config.address);

    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineIdentity {
    name: String,
    vendor: String,
    serial: u32,
    last_active: DateTime<Utc>,
}

async fn machines(
    State(state): State<Arc<SharedState>>,
) -> Result<Json<Vec<MachineIdentity>>, String> {
    let mut items: Vec<MachineIdentity> = Vec::new();

    for ident in state.machine_registry.load().iter() {
        let uid = MachineIdentificationUnique::from_u64(*ident);
        let name = if let Some(v) = state.machine_specs.get(&uid.ident) {
            &v.name
        } else { "undefined" };

        items.push(MachineIdentity {
            name: name.into(),
            vendor: "QiTech GmbH".into(),
            serial: uid.serial,
            last_active: Utc::now(),
        });
    }

    Ok(axum::Json(items))
}

fn ident_from_name(state: &SharedState, name: &str) -> Option<MachineIdentification> {
    for (ident, spec) in state.machine_specs.iter() {
        if spec.name == name {
            return Some(*ident);
        }
    }

    return None;
}

#[derive(Debug, Serialize, Deserialize, Row)]
struct MachineInfoRow<T> {
    name: String,
    value: T,

    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    last_changed: DateTime<Utc>,
}

type Tree = serde_json::Map<String, serde_json::Value>;

fn insert(tree: &mut Map<String, Value>, key: &str, value: Value) {
    let mut parts = key.split('.');
    let first = parts.next().unwrap();

    if let Some(rest) = parts.next() {
        let entry = tree
            .entry(first.to_string())
            .or_insert_with(|| Value::Object(Map::new()));

        let obj = entry.as_object_mut().unwrap();
        insert(obj, rest, value);
    } else {
        tree.insert(first.to_string(), value);
    }
}

async fn machine(
    State(state): State<Arc<SharedState>>,
    Path((name, serial)): Path<(String, u32)>,
) -> Result<Json<Tree>, String> {
    let Some(ident) = ident_from_name(&state, &name) else {
        return Err(format!("No such machine: {name}"));
    };

    let uid = MachineIdentificationUnique {
        ident,
        serial,
    }.as_u64();
    
    let query = r#"
        SELECT
            name,
            argMax(value, ts) AS value,
            max(ts) AS last_changed
        FROM properties_float
        WHERE ident = ?
        GROUP BY name
    "#;

    let query = state.client
        .query(query)
        .bind(uid);

    let rows = query
        .fetch_all::<MachineInfoRow<f64>>()
        .await
        .map_err(|e| e.to_string())?;

    let mut out: Map<String, Value> = Map::new();

    for row in rows {
        insert(&mut out, &row.name, Value::from(row.value));
    }

    Ok(Json(out))
}

async fn property(
    State(state): State<Arc<SharedState>>,
    Path((machine_name, serial, property_name)): Path<(String, u32, String)>,
    Query(query): Query<PropertyQuery>,
) -> Result<Json<Vec<Sample<f64>>>, String> {
    let limit = query.limit.unwrap_or(1000);
    let from = query.from;
    let to = query.to;

    let format = query.format.unwrap_or(ResponseFormat::Json);
    // TODO: implement formatted exports

    let Some(ident) = ident_from_name(&state, &machine_name) else {
        return Err(format!("No such machine: {machine_name}"));
    };

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    Json,
    Binary,
}

#[derive(Debug, Deserialize)]
pub struct PropertyQuery {
    /// Maximum number of samples to return.
    pub limit: Option<u64>,

    /// Start timestamp (e.g. unix ns/ms/s depending on your API).
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub from: Option<DateTime<Utc>>,

    /// End timestamp.
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub to: Option<DateTime<Utc>>,

    /// Response format.
    pub format: Option<ResponseFormat>,
}



#[derive(Debug, Serialize, Deserialize, Row)]
struct Sample<T> {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    ts: DateTime<Utc>,
    value: T,
}

// #[derive(Machine("laser_v1"))] -> generate trait for mutations LaserV1Mutations

// pub fn unpack_uid(value: u64) -> Self {
//     let vendor = (value >> 48) as u16;
//     let machine = (value >> 32) as u16;
//     let serial = value as u32;
// }

/* 
async fn property(
    State(state): State<Arc<SharedState>>,
    Path((machine_uid, property_name)): Path<(u64, String)>,
) -> Result<Json<PropertyHistoryResponse>, String> {
    let query = r#"
        SELECT value
        FROM properties_float
        WHERE ident = ?
        AND name = ?
        ORDER BY ts DESC
        LIMIT 1000
    "#;

    let registry = state.machine_specs;

    let data_type = match registry.get_data_type(machine_uid, &property_name) {
        Ok(v) => v,
        Err(e) => return Err(e.into()),
    };
    
    let query = state.client
        .query(query)
        .bind(machine_uid)
        .bind(&property_name);

    match data_type {
        PropertyType::Float => {
            let samples = extract_rows::<f64>(query).await?;

            Ok(Json(PropertyHistoryResponse {
                machine_id: machine_uid,
                property_name,
                samples: Samples::Float(samples),
            }))
        }

        PropertyType::Integer => {
            let samples = extract_rows::<i64>(query).await?;

            Ok(Json(PropertyHistoryResponse {
                machine_id: machine_uid,
                property_name,
                samples: Samples::Integer(samples),
            }))
        }
        
        PropertyType::Boolean => {
            let samples = extract_rows::<bool>(query).await?;

            Ok(Json(PropertyHistoryResponse {
                machine_id: machine_uid,
                property_name,
                samples: Samples::Boolean(samples),
            }))
        }

        PropertyType::String => {
            let samples = extract_rows::<String>(query).await?;

            Ok(Json(PropertyHistoryResponse {
                machine_id: machine_uid,
                property_name,
                samples: Samples::String(samples),
            }))
        }
    }
}
    */