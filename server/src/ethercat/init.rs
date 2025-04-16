use super::r#loop::setup_loop;
use crate::{
    app_state::{AppState, APP_STATE},
    socketio::main_namespace::{
        ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent, MainNamespaceEvents,
    },
};
use control_core::{
    ethercat::interface_discovery::discover_ethercat_interface,
    socketio::namespace::NamespaceCacheingLogic,
};
use std::sync::Arc;
use thread_priority::{ThreadBuilderExt, ThreadPriority};

pub fn init_ethercat(app_state: Arc<AppState>) {
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
    let interface = smol::block_on(async {
        loop {
            match discover_ethercat_interface().await {
                Ok(interface) => {
                    log::info!("Found working interface: {}", interface);
                    break interface;
                }
                Err(_) => {
                    log::warn!("No working interface found, retrying...");
                    // wait 5 seconds before retrying
                    smol::Timer::after(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    });

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
    tokio::spawn(async move {
        std::thread::Builder::new()
            .name("EthercatThread".to_owned())
            .spawn_with_priority(ThreadPriority::Max, move |_| {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                runtime.block_on(async {
                    log::info!("Starting Ethercat PDU loop");
                    let result = setup_loop(&interface, app_state.clone()).await;
                    if let Err(e) = result {
                        panic!("Failed to setup Ethercat: {:?}", e);
                    } else {
                        log::info!("Ethercat loop exited");
                    }
                })
            })
            .unwrap();
    });
}
