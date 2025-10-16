use std::sync::Arc;

use axum::{Json, body::Body, extract::State, http::Response};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::rest::util::ResponseUtil;
use crate::socketio::main_namespace::{MainNamespaceEvents, machines_event::MachinesEventBuilder};
use control_core::socketio::namespace::NamespaceCacheingLogic;

#[derive(Debug, Serialize)]
pub struct MachineApiEnabledResponse {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct MachineApiEnabledRequest {
    pub enabled: bool,
}

#[axum::debug_handler]
pub async fn get_machine_api_enabled(State(app_state): State<Arc<AppState>>) -> Response<Body> {
    ResponseUtil::ok(MachineApiEnabledResponse {
        enabled: app_state.is_machine_api_enabled(),
    })
}

#[axum::debug_handler]
pub async fn post_machine_api_enabled(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<MachineApiEnabledRequest>,
) -> Response<Body> {
    app_state.set_machine_api_enabled(body.enabled);

    {
        let event = MachinesEventBuilder().build(app_state.clone());
        let mut namespaces_guard = app_state.socketio_setup.namespaces.write().await;
        namespaces_guard
            .main_namespace
            .emit(MainNamespaceEvents::MachinesEvent(event));
    }

    ResponseUtil::ok(MachineApiEnabledResponse {
        enabled: body.enabled,
    })
}
