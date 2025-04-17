use super::r#loop::setup_loop;
use crate::{
    app_state::{AppState, APP_STATE},
    socketio::main_namespace::{
        ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent, MainNamespaceEvents,
    },
};
use anyhow::anyhow;
use control_core::{
    ethercat::interface_discovery::discover_ethercat_interface,
    socketio::namespace::NamespaceCacheingLogic,
};
use std::sync::Arc;
use thread_priority::{ThreadBuilderExt, ThreadPriority};

pub async fn init_ethercat(app_state: Arc<AppState>) -> Result<(), anyhow::Error> {
    // Notify client via socketio
    let _ = smol::block_on(async move {
        let main_namespace = &mut APP_STATE
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = EthercatInterfaceDiscoveryEvent::Discovering(true).build();
        main_namespace.emit_cached(MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(event));
    });

    // tries to find a suitable interface in a loop
    let interface = loop {
        match discover_ethercat_interface().await {
            Ok(interface) => {
                log::info!("Found working interface: {}", interface);
                break interface;
            }
            Err(_) => {
                log::warn!("No working interface found, retrying...");
                // wait 1 seconds before retrying
                smol::Timer::after(std::time::Duration::from_secs(1)).await;
            }
        }
    };

    // Notify client via socketio
    let interface_clone = interface.clone();
    let _ = smol::block_on(async move {
        let main_namespace = &mut APP_STATE
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = EthercatInterfaceDiscoveryEvent::Done(interface_clone).build();
        main_namespace.emit_cached(MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(event));
    });

    // start the event loop
    let loop_thread_handle = std::thread::Builder::new()
        .name("EthercatSetupLoopThread".to_owned())
        .spawn_with_priority(ThreadPriority::Max, move |_| {
            let rt = smol::LocalExecutor::new();
            smol::block_on(rt.run(async move {
                log::info!("Starting Ethercat PDU loop");
                let error = setup_loop(&interface, app_state.clone()).await;
                log::error!("Ethercat PDU loop error: {}", error.unwrap_err());
            }));
        });

    match loop_thread_handle {
        Ok(_) => {}
        Err(err) => {
            return Err(anyhow!("Ethercat loop thread couldn't be created: {}", err));
        }
    };

    Ok(())
}
