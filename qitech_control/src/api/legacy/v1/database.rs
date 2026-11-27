//! `GET /api/v1/database/storage`: how much disk the ClickHouse database uses.
//!
//! Everything comes from ClickHouse's part metadata (`system.parts`, `system.disks`),
//! never from scanning the tables, so it answers instantly even with billions of rows.

use axum::extract::State;
use serde::Deserialize;
use serde::Serialize;

use super::query::ApiError;
use super::query::ApiResult;
use crate::api::db::DATABASE;
use crate::api::legacy::LegacyApiState;
use crate::api::legacy::response_util::ResponseUtil;
use crate::demo::backfill;

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct TableStorage {
    pub table: String,
    pub rows: u64,
    pub bytes_on_disk: u64,
    pub compressed_bytes: u64,
    pub uncompressed_bytes: u64,
    pub parts: u64,
}

/// One daily partition of `machine_measurements`.
#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct DayStorage {
    /// `YYYYMMDD`
    pub day: String,
    pub rows: u64,
    pub bytes_on_disk: u64,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct DatabaseStorage {
    pub database: String,
    pub bytes_on_disk: u64,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct DiskStorage {
    pub name: String,
    pub path: String,
    pub free_space: u64,
    pub total_space: u64,
}

#[derive(Debug, Serialize)]
pub struct DemoStatus {
    pub enabled: bool,
    pub backfill: Option<backfill::Status>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    /// `control_hub` tables, largest first.
    pub tables: Vec<TableStorage>,
    /// Measurements per day, oldest first.
    pub days: Vec<DayStorage>,
    /// Every database on the server, incl. ClickHouse's own `system` logs.
    pub databases: Vec<DatabaseStorage>,
    pub disks: Vec<DiskStorage>,
    /// Deletes still rewriting parts; their space is not free yet.
    pub pending_mutations: u64,
    /// How many measurement parts already carry the pre-aggregated graph data
    /// (projections). Parts written before an update are filled in the background; until
    /// then long graph ranges read the raw rows and are slower.
    pub projections: ProjectionStatus,
    pub demo: DemoStatus,
}

#[derive(Debug, Deserialize, clickhouse::Row)]
struct Count {
    n: u64,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
pub struct ProjectionStatus {
    pub parts_total: u64,
    pub parts_ready: u64,
}

pub async fn storage(State(state): State<LegacyApiState>) -> ApiResult {
    let db = &state.db;

    let tables = db
        .query(
            "SELECT table,
                    sum(rows)                    AS rows,
                    sum(bytes_on_disk)           AS bytes_on_disk,
                    sum(data_compressed_bytes)   AS compressed_bytes,
                    sum(data_uncompressed_bytes) AS uncompressed_bytes,
                    count()                      AS parts
             FROM system.parts
             WHERE active AND database = ?
             GROUP BY table
             ORDER BY bytes_on_disk DESC",
        )
        .bind(DATABASE)
        .fetch_all::<TableStorage>();

    let days = db
        .query(
            "SELECT partition AS day,
                    sum(rows)          AS rows,
                    sum(bytes_on_disk) AS bytes_on_disk
             FROM system.parts
             WHERE active AND database = ? AND table = 'machine_measurements'
             GROUP BY partition
             ORDER BY partition",
        )
        .bind(DATABASE)
        .fetch_all::<DayStorage>();

    let databases = db
        .query(
            "SELECT database, sum(bytes_on_disk) AS bytes_on_disk
             FROM system.parts
             WHERE active
             GROUP BY database
             ORDER BY bytes_on_disk DESC",
        )
        .fetch_all::<DatabaseStorage>();

    let disks = db
        .query("SELECT name, path, free_space, total_space FROM system.disks ORDER BY name")
        .fetch_all::<DiskStorage>();

    let mutations = db
        .query(
            "SELECT count() AS n FROM system.mutations
             WHERE database = ? AND NOT is_done AND command ILIKE '%DELETE%'",
        )
        .bind(DATABASE)
        .fetch_one::<Count>();

    let projections = db
        .query(
            "SELECT count() AS parts_total,
                    countIf(has(projections, 'p_5s') AND has(projections, 'p_1m')) AS parts_ready
             FROM system.parts
             WHERE active AND database = ? AND table = 'machine_measurements'",
        )
        .bind(DATABASE)
        .fetch_one::<ProjectionStatus>();

    let (tables, days, databases, disks, mutations, projections) =
        tokio::try_join!(tables, days, databases, disks, mutations, projections)
            .map_err(ApiError::db("storage query"))?;

    Ok(ResponseUtil::ok(Response {
        tables,
        days,
        databases,
        disks,
        pending_mutations: mutations.n,
        projections,
        demo: DemoStatus {
            enabled: state.demo.is_some(),
            backfill: state.demo.as_ref().map(|d| d.backfill().status()),
        },
    }))
}

/// Tables holding rows per machine `identity`.
const MACHINE_TABLES: [&str; 5] = [
    "machine_measurements",
    "machine_events",
    "machine_config_property_records",
    "machine_state_property_records",
    "machine_markers",
];

#[derive(Debug, Serialize)]
pub struct ClearResponse {
    pub success: bool,
}

/// `POST /api/v1/database/demo/clear`: deletes all demo machine data. Refused while the
/// demo runs, since it would write new rows right away. ClickHouse frees the space in
/// the background (the deletes are mutations that rewrite the affected parts).
pub async fn clear_demo(State(state): State<LegacyApiState>) -> ApiResult {
    if state.demo.is_some() {
        return Err(ApiError::BadRequest(
            "turn the demo machines off before deleting their data",
        ));
    }
    let identities = crate::demo::identities();
    for table in MACHINE_TABLES {
        state
            .db
            .query(&format!("ALTER TABLE {table} DELETE WHERE identity IN ?"))
            .bind(&identities)
            .execute()
            .await
            .map_err(ApiError::db("deleting demo data"))?;
    }
    tracing::info!("demo data deletion queued");
    Ok(ResponseUtil::ok(ClearResponse { success: true }))
}
