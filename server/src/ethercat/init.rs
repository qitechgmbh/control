use super::r#loop::setup_loop;
use crate::{
    app_state::{AppState, APP_STATE},
    panic::PanicDetails,
    socketio::main_namespace::{
        ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent, MainNamespaceEvents,
    },
};
use control_core::{
    ethercat::interface_discovery::discover_ethercat_interface,
    socketio::namespace::NamespaceCacheingLogic,
};
use smol::channel::Sender;
use std::sync::Arc;

pub fn init_ethercat(
    thread_panic_tx: Sender<PanicDetails>,
    app_state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
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
                    // wait 1 seconds before retrying
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

    smol::block_on(setup_loop(thread_panic_tx, &interface, app_state.clone()))?;

    Ok(())
}
