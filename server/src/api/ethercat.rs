use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
};
use serde_json::json;

use crate::app_state::AppState;

use super::util::build_error_response;

#[axum::debug_handler]
pub async fn get_ethercat(State(state): State<Arc<AppState>>) -> Response<Body> {
    let maindevice_guard = state.as_ref().maindevice.read().await;
    let maindevice = match maindevice_guard.as_ref() {
        Some(device) => device,
        None => return build_error_response("MainDevice not initialized"),
    };

    let mut group_guard = state.group.write().await;
    let group = match group_guard.as_mut() {
        Some(group) => group,
        None => return build_error_response("SubDeviceGroup not initialized"),
    };

    let body = json!({
        "devices": (*group).iter(&maindevice).map(|subdevice| {
            json!({
                "address": subdevice.configured_address(),
                "name": subdevice.name(),
            })
        }).collect::<Vec<_>>()
    });

    return Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
}
