//! Graph markers stored in ClickHouse (`machine_markers`), shared by every UI client.
//!
//! - `POST /api/v1/machine/markers`        list the markers of a machine
//! - `POST /api/v1/machine/markers/add`    add one, returns it with its id
//! - `POST /api/v1/machine/markers/remove` remove one by id
//! - `POST /api/v1/machine/markers/clear`  remove all of a machine

use axum::Json;
use axum::extract::State;
use axum::response::Response as AxumResponse;
use qitech_framework::MachineInstanceIdentification;
use serde::Deserialize;
use serde::Serialize;

use crate::api::legacy::LegacyApiState;
use crate::api::legacy::response_util::ResponseUtil;
use crate::api::legacy::types::LegacyMachineIdentificationUnique;

/// Upper bound on markers returned per machine.
const MAX_MARKERS: u64 = 1_000;
const MAX_NAME_LEN: usize = 128;
const MAX_COLOR_LEN: usize = 32;
const DEFAULT_COLOR: &str = "#000000";

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct Marker {
    pub id: String,
    /// Unix ms.
    pub timestamp: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct ListRequest {
    pub machine_identification_unique: LegacyMachineIdentificationUnique,
}

#[derive(Debug, Deserialize)]
pub struct AddRequest {
    pub machine_identification_unique: LegacyMachineIdentificationUnique,
    /// Unix ms.
    pub timestamp: i64,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemoveRequest {
    pub machine_identification_unique: LegacyMachineIdentificationUnique,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct ClearRequest {
    pub machine_identification_unique: LegacyMachineIdentificationUnique,
}

fn identity(machine: LegacyMachineIdentificationUnique) -> u64 {
    MachineInstanceIdentification::from(machine).to_u64()
}

fn db_error(action: &str, e: clickhouse::error::Error) -> AxumResponse {
    tracing::error!(error = %e, "marker {action} failed");
    ResponseUtil::error(&format!("marker {action} failed: {e}"))
}

pub async fn list(
    State(state): State<LegacyApiState>,
    Json(body): Json<ListRequest>,
) -> AxumResponse {
    let identity = identity(body.machine_identification_unique);
    let result = state
        .db
        .query(
            "SELECT toString(id) AS id,
                    toUnixTimestamp64Milli(timestamp) AS timestamp,
                    name,
                    color
             FROM machine_markers
             WHERE identity = ?
             ORDER BY timestamp DESC
             LIMIT ?",
        )
        .bind(identity)
        .bind(MAX_MARKERS)
        .fetch_all::<Marker>()
        .await;

    match result {
        Ok(mut markers) => {
            markers.reverse();
            ResponseUtil::ok(markers)
        }
        Err(e) => db_error("list", e),
    }
}

pub async fn add(
    State(state): State<LegacyApiState>,
    Json(body): Json<AddRequest>,
) -> AxumResponse {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return ResponseUtil::error("marker name must not be empty");
    }
    if name.chars().count() > MAX_NAME_LEN {
        return ResponseUtil::error("marker name is too long");
    }
    if body.timestamp < 0 {
        return ResponseUtil::error("marker timestamp must not be negative");
    }
    let color = match body.color.as_deref().map(str::trim) {
        None | Some("") => DEFAULT_COLOR.to_string(),
        Some(c) if c.len() <= MAX_COLOR_LEN => c.to_string(),
        Some(_) => return ResponseUtil::error("marker color is too long"),
    };

    let identity = identity(body.machine_identification_unique);

    // Generate the id first so the response can carry it.
    let id = match state
        .db
        .query("SELECT toString(generateUUIDv4())")
        .fetch_one::<String>()
        .await
    {
        Ok(id) => id,
        Err(e) => return db_error("add", e),
    };

    let result = state
        .db
        .query(
            "INSERT INTO machine_markers (id, identity, timestamp, name, color)
             SELECT toUUID(?), ?, fromUnixTimestamp64Milli(?), ?, ?",
        )
        .bind(&id)
        .bind(identity)
        .bind(body.timestamp)
        .bind(&name)
        .bind(&color)
        .execute()
        .await;

    match result {
        Ok(()) => ResponseUtil::ok(Marker {
            id,
            timestamp: body.timestamp,
            name,
            color,
        }),
        Err(e) => db_error("add", e),
    }
}

pub async fn remove(
    State(state): State<LegacyApiState>,
    Json(body): Json<RemoveRequest>,
) -> AxumResponse {
    let identity = identity(body.machine_identification_unique);
    let result = state
        .db
        // An unparsable id becomes the zero UUID, which matches nothing.
        .query("DELETE FROM machine_markers WHERE identity = ? AND id = toUUIDOrZero(?)")
        .bind(identity)
        .bind(&body.id)
        .execute()
        .await;

    match result {
        Ok(()) => ResponseUtil::ok(()),
        Err(e) => db_error("remove", e),
    }
}

pub async fn clear(
    State(state): State<LegacyApiState>,
    Json(body): Json<ClearRequest>,
) -> AxumResponse {
    let identity = identity(body.machine_identification_unique);
    let result = state
        .db
        .query("DELETE FROM machine_markers WHERE identity = ?")
        .bind(identity)
        .execute()
        .await;

    match result {
        Ok(()) => ResponseUtil::ok(()),
        Err(e) => db_error("clear", e),
    }
}
