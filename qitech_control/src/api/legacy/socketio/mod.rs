use socketioxide::ParserConfig;
use socketioxide::SocketIoBuilder;
use socketioxide::extract::SocketRef;
use socketioxide::layer::SocketIoLayer;

use crate::api::legacy::LegacySharedState;
use crate::api::types::SharedState;

mod dispatcher;
pub use dispatcher::SocketIODispatcher;

mod events;

mod main_namespace;
pub use main_namespace::MainNamespaceManager;

mod machine_namespace;
pub use machine_namespace::MachineNamespaceManager;

pub fn init(state: SharedState, state_legacy: LegacySharedState) -> SocketIoLayer {
    _ = state;

    let (layer, io) = SocketIoBuilder::new()
        .max_buffer_size(1024)
        .with_parser(ParserConfig::msgpack())
        .build_layer();

    // --- register main namespace ---
    let mut state_main = state_legacy.clone();
    io.ns("/main", move |socket: SocketRef| async move {
        state_main.ns_main.update(|ns| ns.add_socket(socket));
    });

    // --- register machine namespace(s) ---
    let mut state_machines = state_legacy.clone();

    io.dyn_ns(
        "/machine/{vendor}/{machine}/{serial}",
        move |socket: SocketRef| async move {
            state_machines
                .ns_machines
                .update(|ns| ns.add_socket(socket));
        },
    )
    .expect("?");

    // --- yield the layer ---
    layer
}
