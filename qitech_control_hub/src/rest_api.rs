use std::{fmt::Debug, io, sync::Arc};
use axum::{Json, extract::{Path, State}, routing::get};
use chrono::{DateTime, Utc};
use clickhouse::{Row, query::Query};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{SharedState, PropertyType};

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
        .route("/properties/{machine_id}/{property_name}", get(property))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(&config.address)
        .await
        .unwrap();

    println!("[RestApi] Listening on {}", &config.address);

    axum::serve(listener, app).await?;
    Ok(())
}

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

    let registry = state.registry.read().await;

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

async fn extract_rows<T: 'static + DeserializeOwned>(
    query: Query
) -> Result<Vec<PropertyRow<T>>, String> {
    query
        .fetch_all::<PropertyRow<T>>()
        .await
        .map_err(|e| e.to_string())
}
