use super::setup::setup_loop;
use crate::{
    app_state::AppState,
    panic::PanicDetails,
    socketio::main_namespace::{
        MainNamespaceEvents, ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent,
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
    let app_state_clone = app_state.clone();
    let _ = smol::block_on(async move {
        let main_namespace = &mut app_state_clone
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = EthercatInterfaceDiscoveryEvent::Discovering(true).build();
        main_namespace.emit(MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(event));
    });

    // tries to find a suitable interface in a loop
    let interface = smol::block_on(async {
        loop {
            match discover_ethercat_interface().await {
                Ok(interface) => {
                    tracing::info!("Found working interface: {}", interface);
                    break interface;
                }
                Err(_) => {
                    tracing::warn!("No working interface found, retrying...");
                    // wait 1 seconds before retrying
                    smol::Timer::after(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    });

    // Notify client via socketio
    let interface_clone = interface.clone();
    let app_state_clone = app_state.clone();
    let _ = smol::block_on(async move {
        let main_namespace = &mut app_state_clone
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;
        let event = EthercatInterfaceDiscoveryEvent::Done(interface_clone).build();
        main_namespace.emit(MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(event));
    });

    smol::block_on(setup_loop(thread_panic_tx, &interface, app_state.clone()))?;

    Ok(())
}
