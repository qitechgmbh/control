mod types;
use types::EtherCATDeviceMetadata;
use types::LegacyMachineIdentificationUnique;
use types::ModbusDeviceMetadata;

mod response_util;
mod socketio;
use socketio::MachineNamespaceManager;
use socketio::MainNamespaceManager;
pub use socketio::SocketIODispatcher;
pub use socketio::init as init_socket_io;
#[cfg(test)]
pub use socketio::legacy_adapter_events;

pub mod v1;
pub mod v2;

use qitech_framework::MachineInstanceIdentification;
use qitech_framework_hub::ActorContext;
use tokio::sync::mpsc;

use crate::api::types::Swappable;
use crate::demo::DemoHandle;

mod adapter;
use adapter::MachineLegacyDataAdapter;

/// State handed to every v1/v2 handler.
#[derive(Clone)]
pub struct LegacyApiState {
    pub ctx: ActorContext,
    pub machines_dirty_tx: mpsc::Sender<MachineInstanceIdentification>,
    pub legacy: LegacySharedState,
    /// ClickHouse client (database `control_hub`) for history queries.
    pub db: clickhouse::Client,
    /// Set while the demo machines run (`DEMO_MACHINES=true`).
    pub demo: Option<DemoHandle>,
}

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
