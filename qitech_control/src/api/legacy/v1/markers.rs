//! Graph markers stored in ClickHouse (`machine_markers`), shared by every UI client.
//!
//! - `POST /api/v1/machine/markers`        list the markers of a machine
//! - `POST /api/v1/machine/markers/add`    add one, returns it with its id
//! - `POST /api/v1/machine/markers/remove` remove one by id
//! - `POST /api/v1/machine/markers/clear`  remove all of a machine

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde::Serialize;

use super::query::ApiError;
use super::query::ApiResult;
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

/// Body of `list` and `clear`.
#[derive(Debug, Deserialize)]
pub struct MachineRequest {
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

pub async fn list(
    State(state): State<LegacyApiState>,
    Json(body): Json<MachineRequest>,
) -> ApiResult {
    let mut markers = state
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
        .bind(body.machine_identification_unique.identity())
        .bind(MAX_MARKERS)
        .fetch_all::<Marker>()
        .await
        .map_err(ApiError::db("marker list"))?;

    // newest `MAX_MARKERS`, returned oldest first
    markers.reverse();
    Ok(ResponseUtil::ok(markers))
}

pub async fn add(State(state): State<LegacyApiState>, Json(body): Json<AddRequest>) -> ApiResult {
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("marker name must not be empty"));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(ApiError::BadRequest("marker name is too long"));
    }
    if body.timestamp < 0 {
        return Err(ApiError::BadRequest(
            "marker timestamp must not be negative",
        ));
    }
    let color = match body.color.as_deref().map(str::trim) {
        None | Some("") => DEFAULT_COLOR.to_string(),
        Some(c) if c.len() <= MAX_COLOR_LEN => c.to_string(),
        Some(_) => return Err(ApiError::BadRequest("marker color is too long")),
    };

    let marker = Marker {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: body.timestamp,
        name,
        color,
    };
    state
        .db
        .query(
            "INSERT INTO machine_markers (id, identity, timestamp, name, color)
             SELECT toUUID(?), ?, fromUnixTimestamp64Milli(?), ?, ?",
        )
        .bind(&marker.id)
        .bind(body.machine_identification_unique.identity())
        .bind(marker.timestamp)
        .bind(&marker.name)
        .bind(&marker.color)
        .execute()
        .await
        .map_err(ApiError::db("marker add"))?;

    Ok(ResponseUtil::ok(marker))
}

pub async fn remove(
    State(state): State<LegacyApiState>,
    Json(body): Json<RemoveRequest>,
) -> ApiResult {
    state
        .db
        // An unparsable id becomes the zero UUID, which matches nothing.
        .query("DELETE FROM machine_markers WHERE identity = ? AND id = toUUIDOrZero(?)")
        .bind(body.machine_identification_unique.identity())
        .bind(&body.id)
        .execute()
        .await
        .map_err(ApiError::db("marker remove"))?;
    Ok(ResponseUtil::ok(()))
}

pub async fn clear(
    State(state): State<LegacyApiState>,
    Json(body): Json<MachineRequest>,
) -> ApiResult {
    state
        .db
        .query("DELETE FROM machine_markers WHERE identity = ?")
        .bind(body.machine_identification_unique.identity())
        .execute()
        .await
        .map_err(ApiError::db("marker clear"))?;
    Ok(ResponseUtil::ok(()))
}
