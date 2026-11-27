mod types;
pub use types::SharedState;

mod legacy;
pub use legacy::LegacySharedState;
pub use legacy::SocketIODispatcher;
pub(crate) use legacy::v1::modbus::broadcast_devices as broadcast_modbus_devices;

mod server;
pub use server::Server;

mod db;
pub use db::ClickhouseWriter;
pub use db::connect as connect_db;
pub(crate) use db::tables as db_tables;
#[cfg(test)]
pub(crate) use legacy::legacy_adapter_events;
