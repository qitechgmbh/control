use crate::metrics::io::set_ethercat_iface;
use control_core::ethercat::interface_discovery::discover_ethercat_interface;
use smol::Timer;
use std::time::Duration;

pub async fn find_ethercat_interface() -> String {
    loop {
        match discover_ethercat_interface().await {
            Ok(interface) => {
                tracing::info!("Found EtherCAT Interface at: {}", interface);
                set_ethercat_iface(&interface);
                return interface;
            }
            Err(e) => {
                tracing::warn!("No working interface found: {}. Retrying...", e);
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    }
}
