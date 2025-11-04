use crate::{
    app_state::AppState,
    rest::util::{ResponseUtil, ResponseUtilError},
};
use axum::{Json, body::Body, extract::State, http::Response};
use control_core::{
    machines::connection::MachineConnection,
    rest::mutation::{MachineEventBody, MachineEventResponse},
};
use std::sync::Arc;

#[axum::debug_handler]
pub async fn post_machine_event(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<MachineEventBody>,
) -> Response<Body> {
    let result = _post_machine_event(State(app_state), Json(body)).await;
    match result {
        Ok(data) => ResponseUtil::ok(MachineEventResponse::success(data)),
        Err(e) => ResponseUtilError::Error(e).into(),
    }
}

async fn _post_machine_event(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<MachineEventBody>,
) -> Result<serde_json::Value, anyhow::Error> {
    // Check if read-only API is enabled
    let read_only_enabled = app_state
        .read_only_api_enabled
        .load(std::sync::atomic::Ordering::Relaxed);
    if !read_only_enabled {
        return Err(anyhow::anyhow!(
            "Read-only API is disabled. Enable it in the configuration to use this endpoint."
        ));
    }

    // Get machine
    let machines = app_state.machines.read().await;
    let machine_slot = machines.get(&body.machine_identification_unique);

    let machine_slot = match machine_slot {
        Some(slot) => slot,
        None => {
            return Err(anyhow::anyhow!(
                "[{}::_post_machine_event] Machine not found",
                module_path!()
            ));
        }
    };

    // Lock the slot to access the machine connection
    let slot_guard = machine_slot.lock().await;

    let machine = match &slot_guard.machine_connection {
        MachineConnection::Connected(machine) => machine,
        MachineConnection::Disconnected => {
            return Err(anyhow::anyhow!(
                "[{}::_post_machine_event] Machine is disconnected",
                module_path!(),
            ));
        }
        MachineConnection::Error(e) => {
            return Err(anyhow::anyhow!(
                "[{}::_post_machine_event] Machine connection error: {}",
                module_path!(),
                e
            ));
        }
    };

    // Log
    tracing::info!(
        "Querying machine events (read-only) machine={}",
        body.machine_identification_unique,
    );
    let span = tracing::info_span!("machine_event", machine = %body.machine_identification_unique);
    let _span = span.enter();

    // Lock the machine and call api_event
    let mut machine_guard = machine.lock().await;

    // Call api_event with the requested event fields
    machine_guard.api_event(body.events.as_ref()).map_err(|e| {
        anyhow::anyhow!(
            "[{}::_post_machine_event] Machine api_event error: {}",
            module_path!(),
            e
        )
    })
}
