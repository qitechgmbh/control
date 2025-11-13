use std::sync::Arc;

use control_core::socketio::namespace::NamespaceCacheingLogic;

use crate::{
    app_state::SharedState,
    socketio::main_namespace::{
        MainNamespaceEvents, ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent,
    },
};

pub async fn send_ethercat_discovering(app_state: Arc<SharedState>) {
    {
        let main_namespace = &mut app_state
            .socketio_setup
            .namespaces
            .write()
            .await
            .main_namespace;

        let event = EthercatInterfaceDiscoveryEvent::Discovering(true).build();
        main_namespace.emit(MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(event));
    }
}
pub async fn send_ethercat_found(app_state: Arc<SharedState>, interface: &str) {
    // Notify client via socketio
    let interface = interface.to_string();
    let app_state_socketio = app_state.clone();
    let main_namespace = &mut app_state_socketio
        .socketio_setup
        .namespaces
        .write()
        .await
        .main_namespace;

    let event = EthercatInterfaceDiscoveryEvent::Done(interface).build();
    main_namespace.emit(MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(event));
}
