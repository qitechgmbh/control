use crate::{
    app_state::AppState,
    rest::util::ResponseUtil,
    socketio::main_namespace::{
        MainNamespaceEvents, read_only_api_status_event::ReadOnlyApiStatusEventBuilder,
    },
};
use axum::{Json, body::Body, extract::State, http::Response};
use control_core::rest::mutation::MutationResponse;
use control_core::socketio::namespace::NamespaceCacheingLogic;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ReadOnlyApiConfigBody {
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct ReadOnlyApiStatusResponse {
    pub enabled: bool,
}

#[axum::debug_handler]
pub async fn post_read_only_api_config(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<ReadOnlyApiConfigBody>,
) -> Response<Body> {
    tracing::info!(
        "Received read-only API config request: enabled={}",
        body.enabled
    );

    app_state
        .read_only_api_enabled
        .store(body.enabled, std::sync::atomic::Ordering::Relaxed);

    tracing::info!("Read-only API enabled: {}", body.enabled);

    // Emit event to notify clients
    let event_builder = ReadOnlyApiStatusEventBuilder();
    let event = event_builder.build(app_state.clone());

    tracing::debug!("Attempting to acquire write lock on namespaces");
    let mut namespaces = app_state.socketio_setup.namespaces.write().await;
    tracing::debug!("Successfully acquired write lock on namespaces");

    namespaces
        .main_namespace
        .emit(MainNamespaceEvents::ReadOnlyApiStatusEvent(event));

    tracing::info!("Read-only API event emitted successfully");

    ResponseUtil::ok(MutationResponse::success())
}

#[axum::debug_handler]
pub async fn get_read_only_api_status(State(app_state): State<Arc<AppState>>) -> Response<Body> {
    let enabled = app_state
        .read_only_api_enabled
        .load(std::sync::atomic::Ordering::Relaxed);

    ResponseUtil::ok(ReadOnlyApiStatusResponse { enabled })
}
