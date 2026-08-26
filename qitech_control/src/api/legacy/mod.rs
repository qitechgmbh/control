mod types;
use types::EtherCATDeviceMetadata;
use types::MachineIdentificationUnique;

mod socketio;
use socketio::MachineNamespaceManager;
use socketio::MainNamespaceManager;
pub use socketio::SocketIODispatcher;
pub use socketio::init as init_socket_io;

pub mod v1;

use crate::api::types::Swappable;

mod adapter;
use adapter::MachineLegacyDataAdapter;

#[derive(Clone)]
pub struct LegacySharedState {
    ns_main: Swappable<MainNamespaceManager>,
    ns_machines: Swappable<MachineNamespaceManager>,
}

impl LegacySharedState {
    pub fn new() -> Self {
        Self {
            ns_main: Default::default(),
            ns_machines: Default::default(),
        }
    }
}
