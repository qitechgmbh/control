use control_core::ethercat::interface_discovery::discover_ethercat_interface;
use smol::{Task, Timer};
use std::time::Duration;

pub async fn find_ethercat_interface() -> Result<String, anyhow::Error> {
    loop {
        match discover_ethercat_interface().await {
            Ok(interface) => {
                tracing::info!("Found EtherCAT Interface at: {}", interface);
                return Ok(interface);
            }
            Err(e) => {
                tracing::warn!("No working interface found: {}. Retrying...", e);
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    }
}

// Returns a Future to the interface
pub fn start_interface_discovery() -> Task<Result<String, anyhow::Error>> {
    smol::spawn(find_ethercat_interface())
}
