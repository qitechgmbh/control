use crate::{
    ethercat::{
        config::{MAX_SUBDEVICES, PDI_LEN},
        device_identification::{MachineDeviceIdentification, MachineIdentification},
    },
    socketio::room::Rooms,
};
use ethercat_hal::{actors::Actor, devices::Device};
use ethercrab::{subdevice_group::Op, MainDevice, SubDeviceGroup};
use serde::{Deserialize, Serialize};
use socketioxide::SocketIo;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::RwLock;

pub struct AppState {
    pub socketio_rooms: RwLock<Rooms>,
    pub socketio: RwLock<Option<SocketIo>>,
    pub ethercat_setup: Arc<RwLock<Option<EthercatSetup>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineInfo {
    pub machine_identification: MachineIdentification,
    pub error: Option<String>,
}

pub struct EthercatSetup {
    /// High level logical drivers
    /// They read & write to the `devices` / nested actors
    pub actors: Vec<Arc<RwLock<dyn Actor>>>,
    /// Machine Infos
    /// Same length and order as `identified_device_groups`
    /// Validated device groups are machines
    /// If a machine is not valid/complete its an [anyhow::Error]
    pub machine_infos: Vec<MachineInfo>,
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
        machine_infos: Vec<MachineInfo>,
        identified_device_groups: Vec<Vec<MachineDeviceIdentification>>,
        undetected_devices: Vec<MachineDeviceIdentification>,
        devices: Vec<Arc<RwLock<dyn Device>>>,
        group: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
        delays: Vec<Option<u32>>,
        maindevice: MainDevice<'static>,
    ) -> Self {
        Self {
            actors,
            machine_infos,
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
            socketio_rooms: RwLock::new(Rooms::new()),
            socketio: RwLock::new(None),
            ethercat_setup: Arc::new(RwLock::new(None)),
        }
    }
}

pub static APP_STATE: LazyLock<Arc<AppState>> = LazyLock::new(|| Arc::new(AppState::new()));
