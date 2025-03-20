use crate::app_state::APP_STATE;

use super::room::{main::MainRoom, room::RoomInterface, room_id::RoomId};

pub struct Rooms {
    pub main_room: MainRoom,
}

impl Rooms {
    pub fn new() -> Self {
        Self {
            main_room: MainRoom::new(),
        }
    }

    pub async fn apply_mut(
        &mut self,
        room_id: RoomId,
        callback: impl FnOnce(Result<&mut dyn RoomInterface, anyhow::Error>),
    ) {
        match room_id {
            RoomId::Main => callback(Ok(&mut self.main_room.0)),
            RoomId::Machine(machine_identification_unique) => {
                let ethercat_setup_guard = APP_STATE.ethercat_setup.read().await;
                let ethercat_setup_guard = match ethercat_setup_guard.as_ref() {
                    Some(ethercat_setup_guard) => ethercat_setup_guard,
                    None => {
                        callback(Err(anyhow::anyhow!("Ethercat setup not found")));
                        return;
                    }
                };

                // get machine
                let machine = match ethercat_setup_guard
                    .machines
                    .get(&machine_identification_unique)
                {
                    Some(machine) => machine,
                    None => {
                        callback(Err(anyhow::anyhow!(
                            "Machine {} not found",
                            machine_identification_unique
                        )));
                        return;
                    }
                };

                // check if machine has error
                let machine = match machine {
                    Ok(machine) => machine,
                    Err(err) => {
                        callback(Err(anyhow::anyhow!(
                            "Machine {} has error: {}",
                            machine_identification_unique,
                            err
                        )));
                        return;
                    }
                };

                let mut machine_guard = machine.write().await;
                let room = machine_guard.api_event_room();
                callback(Ok(room));
            }
        }
    }

    /// Iterate over all rooms with mutable access using synchronous callbacks
    pub async fn for_each_mut(
        &mut self,
        mut callback: impl FnMut(RoomId, Result<&mut dyn RoomInterface, anyhow::Error>),
    ) {
        // Process main room
        callback(RoomId::Main, Ok(&mut self.main_room.0));

        // Process machine rooms
        let ethercat_setup_guard = APP_STATE.ethercat_setup.read().await;
        let ethercat_setup_guard = match ethercat_setup_guard.as_ref() {
            Some(guard) => guard,
            None => return, // No machine rooms available
        };

        for (machine_id, machine_result) in &ethercat_setup_guard.machines {
            let room_id = RoomId::Machine(machine_id.clone());

            // Check if machine has an error
            let machine = match machine_result {
                Ok(machine) => machine,
                Err(err) => {
                    // Call the callback with the error
                    callback(
                        room_id,
                        Err(anyhow::anyhow!("Machine {} has error: {}", machine_id, err)),
                    );
                    continue;
                }
            };

            // Get the room interface
            let mut machine_guard = machine.write().await;
            let room = machine_guard.api_event_room();
            callback(room_id, Ok(room));
        }
    }
}
