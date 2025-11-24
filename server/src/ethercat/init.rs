use control_core::ethercat::interface_discovery::probe_ethernet;
use smol::{Task, Timer};
use std::time::Duration;

pub async fn find_ethercat_interface() -> String {
    loop {
        let res = probe_ethernet().await;

        if res.ethercat.is_some() {
            return res.ethercat.unwrap();
        }

        tracing::warn!("No working interface found. Retrying...");
        Timer::after(Duration::from_secs(1)).await;
    }
}

// Returns a Future to the interface
pub fn start_interface_discovery() -> Task<String> {
    smol::spawn(find_ethercat_interface())
}
