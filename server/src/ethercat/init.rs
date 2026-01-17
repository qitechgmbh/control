use control_core::ethercat::interface_discovery::discover_ethercat_interface;
use smol::Timer;
use std::{sync::Arc, time::Duration};

use crate::{
    app_state::SharedState,
    ethercat::{ethercat_discovery_info::send_ethercat_found, setup::setup_loop},
    metrics::io::set_ethercat_iface,
};

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

pub async fn start_interface_discovery(app_state: Arc<SharedState>) {
    let interface = find_ethercat_interface().await;

    tracing::info!("Calling setup_loop");
    let res = setup_loop(&interface, app_state.clone()).await;
    match res {
        Ok(_) => tracing::info!("Successfully initialized EtherCAT network"),
        Err(e) => {
            tracing::error!(
                "[{}::main] Failed to initialize EtherCAT network \n{:?}",
                module_path!(),
                e
            );
        }
    }

    send_ethercat_found(app_state, &interface).await;
}
