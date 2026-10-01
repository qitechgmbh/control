use axum::Extension;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use qitech_framework::MachineIdentification;
use serde::Serialize;

use crate::api::legacy::LegacyApiState;
use crate::api::legacy::adapter;
use crate::api::legacy::v2::internal_error;
use crate::api::legacy::v2::MachineEntry;
use crate::api::legacy::v2::Result;
use crate::api::legacy::v2::bad_request;
use crate::api::legacy::v2::json;
use crate::api::legacy::v2::not_found;

// --- get handler ---
#[derive(Serialize, Debug, PartialEq)]
pub struct ResponseGet {
    machine: MachineEntry,
    state: serde_json::Value,
    live_values: serde_json::Value,
}

pub async fn get(
    Extension(id): Extension<MachineIdentification>,
    State(state): State<LegacyApiState>,
    Path(serial): Path<u16>,
) -> Result<ResponseGet> {
    let ident = id.unique(serial);

    let Some(adapter) = adapter::get(ident.machine) else {
        return Err(internal_error("No adapter for this machine available"));
    };

    let ns = state.legacy.ns_machines.read();
    let Some(entry) = ns.registry.get(&ident) else {
        return Err(not_found("no such machine"));
    };

    let Some(state) = (adapter.init_state_event)(
        &entry.instance, 
        false
    ) else {
        return Err(internal_error("Could not create state event"));
    };

    let Some(live_values) = (adapter.init_measurements_event)(
        &entry.instance
    ) else {
        return Err(internal_error("Could not create live values event"));
    };

    json(ResponseGet {
        machine: MachineEntry::from_ident(ident),
        state,
        live_values,
    })
}

pub type RequestPost = Vec<serde_json::Value>;

// --- post handler ---
pub async fn post(
    Extension(id): Extension<MachineIdentification>,
    State(state): State<LegacyApiState>,
    Path(serial): Path<u16>,
    Json(body): Json<RequestPost>,
) -> Result<()> {
    let ident = id.unique(serial);

    let Some(adapter) = adapter::get(ident.machine) else {
        return Err(not_found("no such machine"));
    };

    let mut requests = Vec::new();

    for request in body {
        let mut items = match (adapter.convert_request)(ident, request) {
            Ok(requests) => requests,
            Err(error) => {
                return Err(bad_request(error));
            }
        };

        requests.append(&mut items);
    }

    // Sequential, fail-fast: a compound legacy mutation (e.g. autotune start) may need its writes
    // applied in order before a later request in the batch depends on them.
    for request in requests {
        state.machines_dirty_tx.send(ident).await.expect("pray");

        match state.ctx.send_request(request).await {
            Ok(Ok(())) => {}

            Ok(Err(error)) => {
                return Err(bad_request(error));
            }

            Err(error) => {
                return Err(bad_request(error));
            }
        }
    }

    json(())
}
