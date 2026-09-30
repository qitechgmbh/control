mod types;
use types::EtherCATDeviceMetadata;
<<<<<<< HEAD
use types::LegacyMachineIdentificationUnique;
use types::ModbusDeviceMetadata;

mod response_util;
=======
use types::MachineIdentificationUnique;
use types::ModbusDeviceMetadata;

>>>>>>> 8e49141 (Jse control v2 (#1680))
mod socketio;
use socketio::MachineNamespaceManager;
use socketio::MainNamespaceManager;
pub use socketio::SocketIODispatcher;
pub use socketio::init as init_socket_io;

pub mod v1;
<<<<<<< HEAD
pub mod v2;

use qitech_framework::MachineInstanceIdentification;
use qitech_framework_hub::ActorContext;
use tokio::sync::mpsc;
=======
>>>>>>> 8e49141 (Jse control v2 (#1680))

use crate::api::types::Swappable;

mod adapter;
use adapter::MachineLegacyDataAdapter;

<<<<<<< HEAD
/// State handed to every v1/v2 handler.
#[derive(Clone)]
pub struct LegacyApiState {
    pub ctx: ActorContext,
    pub machines_dirty_tx: mpsc::Sender<MachineInstanceIdentification>,
    pub legacy: LegacySharedState,
}

=======
>>>>>>> 8e49141 (Jse control v2 (#1680))
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
