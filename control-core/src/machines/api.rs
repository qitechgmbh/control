use crate::socketio::room::RoomInterface;
use serde_json::Value;

pub trait MachineApi {
    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error>;
    fn api_event_room(&mut self) -> &mut dyn RoomInterface;
}
