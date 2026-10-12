mod types;
use types::EtherCATDeviceMetadata;
<<<<<<< HEAD
<<<<<<< HEAD
use types::LegacyMachineIdentificationUnique;
use types::ModbusDeviceMetadata;

mod response_util;
=======
use types::MachineIdentificationUnique;
use types::ModbusDeviceMetadata;

>>>>>>> 8e49141 (Jse control v2 (#1680))
=======
use types::LegacyMachineIdentificationUnique;
use types::ModbusDeviceMetadata;

mod response_util;
>>>>>>> e8ebd9a (remove dead code and add api v2 and improve api in general)
mod socketio;
use socketio::MachineNamespaceManager;
use socketio::MainNamespaceManager;
pub use socketio::SocketIODispatcher;
pub use socketio::init as init_socket_io;

pub mod v1;
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> e8ebd9a (remove dead code and add api v2 and improve api in general)
pub mod v2;

use qitech_framework::MachineInstanceIdentification;
use qitech_framework_hub::ActorContext;
use tokio::sync::mpsc;
<<<<<<< HEAD
=======
>>>>>>> 8e49141 (Jse control v2 (#1680))
=======
>>>>>>> e8ebd9a (remove dead code and add api v2 and improve api in general)

use crate::api::types::Swappable;

mod adapter;
use adapter::MachineLegacyDataAdapter;

<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> e8ebd9a (remove dead code and add api v2 and improve api in general)
/// State handed to every v1/v2 handler.
#[derive(Clone)]
pub struct LegacyApiState {
    pub ctx: ActorContext,
    pub machines_dirty_tx: mpsc::Sender<MachineInstanceIdentification>,
    pub legacy: LegacySharedState,
}

<<<<<<< HEAD
=======
>>>>>>> 8e49141 (Jse control v2 (#1680))
=======
>>>>>>> e8ebd9a (remove dead code and add api v2 and improve api in general)
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
