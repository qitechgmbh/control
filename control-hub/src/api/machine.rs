use std::sync::Arc;
use axum::{Json, extract::{Path, State}};
use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use machine_core::MachineIdentificationUnique;
use serde_json::{Map, Value};

use crate::SharedState;

pub async fn handle(
    State(state): State<Arc<SharedState>>,
    Path((name, serial)): Path<(String, u32)>,
) -> Result<Json<Map<String, Value>>, String> {
    println!("wtf");

    let Some(ident) = state.machine_slugs.get(&name) else {
        return Err(format!("No such machine: {name}"));
    };

    let uid = MachineIdentificationUnique {
        ident: *ident,
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
        .fetch_all::<RowEntry<f64>>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(generate_tree(rows)))
}

fn generate_tree(rows: Vec<RowEntry<f64>>) -> Map<String, Value> {
    let mut tree = Map::new();

    for row in rows {
        let mut current: &mut Map<String, Value> = &mut tree;
        let mut parts = row.name.split('.').peekable();

        while let Some(part) = parts.next() {
            let is_last = parts.peek().is_none();

            if is_last {
                // last segment → insert value
                current.insert(part.to_string(), Value::from(row.value));
            } else {
                // intermediate segment → ensure object exists
                current = current
                    .entry(part.to_string())
                    .or_insert_with(|| Value::Object(Map::new()))
                    .as_object_mut()
                    .expect("tree structure corrupted: expected object");
            }
        }
    }

    tree
}

#[derive(Debug, Serialize, Deserialize, Row)]
struct RowEntry<T> {
    name: String,
    value: T,

    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    last_changed: DateTime<Utc>,
}