use std::sync::Arc;

use control_core::socketio::event::Event;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadOnlyApiStatusEvent {
    pub enabled: bool,
    pub ip_addresses: Vec<String>,
}

/// Get all local IPv4 addresses (excluding loopback)
fn get_local_ip_addresses() -> Vec<String> {
    use local_ip_address::list_afinet_netifas;
    use std::net::IpAddr;

    let mut ip_addresses = Vec::new();

    if let Ok(network_interfaces) = list_afinet_netifas() {
        for (_name, ip) in network_interfaces {
            // Only include non-loopback IPv4 addresses
            if !ip.is_loopback() && matches!(ip, IpAddr::V4(_)) {
                ip_addresses.push(ip.to_string());
            }
        }
    }

    // Sort for consistent ordering
    ip_addresses.sort();
    ip_addresses
}

pub struct ReadOnlyApiStatusEventBuilder();

impl ReadOnlyApiStatusEventBuilder {
    const NAME: &'static str = "ReadOnlyApiStatusEvent";

    pub fn build(&self, app_state: Arc<AppState>) -> Event<ReadOnlyApiStatusEvent> {
        let enabled = app_state
            .read_only_api_enabled
            .load(std::sync::atomic::Ordering::Relaxed);

        let ip_addresses = get_local_ip_addresses();

        Event::new(
            Self::NAME,
            ReadOnlyApiStatusEvent {
                enabled,
                ip_addresses,
            },
        )
    }
}
