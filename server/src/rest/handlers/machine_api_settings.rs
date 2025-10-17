use std::sync::Arc;

use axum::{Json, body::Body, extract::State, http::Response};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::rest::util::ResponseUtil;
use crate::socketio::main_namespace::{MainNamespaceEvents, machines_event::MachinesEventBuilder};
use control_core::socketio::namespace::NamespaceCacheingLogic;

fn get_local_ip_addresses() -> Vec<String> {
    use interfaces::Interface;

    Interface::get_all()
        .unwrap_or_default()
        .into_iter()
        .filter(|iface| {
            iface.is_up()
                && iface.is_running()
                && !iface.is_loopback()
                && !iface.name.starts_with("bridge")
                && !iface.name.starts_with("utun")
                && !iface.name.starts_with("awdl")
                && !iface.name.starts_with("anpi")
                && !iface.name.starts_with("llw")
        })
        .flat_map(|iface| {
            iface.addresses.into_iter().filter_map(|addr| {
                // Only include IPv4 addresses
                if addr.addr.is_some() && addr.addr.unwrap().is_ipv4() {
                    Some(addr.addr.unwrap().ip().to_string())
                } else {
                    None
                }
            })
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct MachineApiEnabledResponse {
    pub enabled: bool,
    pub ip_addresses: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MachineApiEnabledRequest {
    pub enabled: bool,
}

#[axum::debug_handler]
pub async fn get_machine_api_enabled(State(app_state): State<Arc<AppState>>) -> Response<Body> {
    ResponseUtil::ok(MachineApiEnabledResponse {
        enabled: app_state.is_machine_api_enabled(),
        ip_addresses: get_local_ip_addresses(),
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
        ip_addresses: get_local_ip_addresses(),
    })
}
