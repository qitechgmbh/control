use crate::{
    ethercat::{
        config::{MAX_SUBDEVICES, PDI_LEN},
        device_identification::{MachineDeviceIdentification, MachineIdentificationUnique},
    },
    machines::Machine,
    socketio::room::{main::MainRoom, room::RoomInterface, room_id::RoomId},
};
use ethercat_hal::{actors::Actor, devices::Device};
use ethercrab::{subdevice_group::Op, MainDevice, SubDeviceGroup};
use socketioxide::SocketIo;
use std::sync::Arc;
use std::{collections::HashMap, sync::LazyLock};
use tokio::sync::RwLock;

pub struct Rooms {
    pub main_room: MainRoom,
}

impl Rooms {
    pub fn new() -> Self {
        Self {
            main_room: MainRoom::new(),
        }
    }

    pub async fn use_mut(
        &mut self,
        ethercat_setup: &mut EthercatSetup,
        room_id: RoomId,
        callback: impl FnOnce(Result<&mut dyn RoomInterface, anyhow::Error>),
    ) {
        match room_id {
            RoomId::Main => callback(Ok(&mut self.main_room.0)),
            RoomId::Machine(machine_identification_unique) => {
                // get machine
                let machine = match ethercat_setup.machines.get(&machine_identification_unique) {
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
}

pub struct SocketioSetup {
    pub socketio: RwLock<Option<SocketIo>>,
    pub rooms: RwLock<Rooms>,
}

pub struct AppState {
    pub socketio_setup: SocketioSetup,
    pub ethercat_setup: Arc<RwLock<Option<EthercatSetup>>>,
}

pub struct EthercatSetup {
    /// High level logical drivers
    /// They read & write to the `devices` / nested actors
    pub actors: Vec<Arc<RwLock<dyn Actor>>>,
    /// Machines
    /// Actual machine interfaces
    pub machines:
        HashMap<MachineIdentificationUnique, Result<Arc<RwLock<dyn Machine>>, anyhow::Error>>,
    /// Metadata about a device groups
    /// Used for the device table in the UI
    pub identified_device_groups: Vec<Vec<MachineDeviceIdentification>>,
    /// Metadata about unidentified devices
    /// Used for the device table in the UI
    pub unidentified_devices: Vec<MachineDeviceIdentification>,
    /// All Ethercat devices
    /// Device-Specific interface for all devices
    /// Same length and order as SubDevices inside `group`
    pub devices: Vec<Arc<RwLock<dyn Device>>>,
    /// All Ethercat devices
    /// Generic interface for all devices
    /// Needed to interface with the devices on an Ethercat level
    pub group: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
    pub delays: Vec<Option<u32>>,
    /// The Ethercat main device
    /// Needed to interface with the devices
    pub maindevice: MainDevice<'static>,
}

impl EthercatSetup {
    pub fn new(
        actors: Vec<Arc<RwLock<dyn Actor>>>,
        machines: HashMap<
            MachineIdentificationUnique,
            Result<Arc<RwLock<dyn Machine>>, anyhow::Error>,
        >,
        identified_device_groups: Vec<Vec<MachineDeviceIdentification>>,
        undetected_devices: Vec<MachineDeviceIdentification>,
        devices: Vec<Arc<RwLock<dyn Device>>>,
        group: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
        delays: Vec<Option<u32>>,
        maindevice: MainDevice<'static>,
    ) -> Self {
        Self {
            actors,
            machines,
            identified_device_groups,
            unidentified_devices: undetected_devices,
            devices,
            group,
            delays,
            maindevice,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            socketio_setup: SocketioSetup {
                socketio: RwLock::new(None),
                rooms: RwLock::new(Rooms::new()),
            },
            ethercat_setup: Arc::new(RwLock::new(None)),
        }
    }
}

pub static APP_STATE: LazyLock<Arc<AppState>> = LazyLock::new(|| Arc::new(AppState::new()));
