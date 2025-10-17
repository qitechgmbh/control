use std::sync::Arc;

use axum::{body::Body, extract::State, http::Response};
use serde::Serialize;

use crate::app_state::AppState;
use crate::rest::util::ResponseUtil;

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
            iface.addresses.clone().into_iter().filter_map(|addr| {
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

#[axum::debug_handler]
pub async fn get_machine_api_enabled(State(app_state): State<Arc<AppState>>) -> Response<Body> {
    ResponseUtil::ok(MachineApiEnabledResponse {
        enabled: app_state.is_machine_api_enabled(),
        ip_addresses: get_local_ip_addresses(),
    })
}
