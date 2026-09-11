mod types;
pub use types::SharedState;

mod legacy;
pub use legacy::LegacySharedState;
pub use legacy::SocketIODispatcher;
pub(crate) use legacy::v1::modbus::broadcast_devices as broadcast_modbus_devices;

mod server;
pub use server::Server;
