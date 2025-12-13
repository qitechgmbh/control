use control_core::ethernet::ethercat_interface_discovery::probe_ethercat;
use smol::Timer;
use std::{sync::Arc, time::Duration};

use crate::{app_state::SharedState, ethercat::{ethercat_discovery_info::send_ethercat_found, setup::setup_loop}};

pub async fn find_ethercat_interface() -> String {
    loop {
        let res = probe_ethercat().await;

        if let Some(interface) = res {
            tracing::info!("Found EtherCAT Interface at: {}", interface);
            return interface;
        }

        tracing::warn!("No working interface found. Retrying...");
        Timer::after(Duration::from_secs(1)).await;
    }
}

pub async fn start_ethercat_discovery(app_state: Arc<SharedState>) {
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
