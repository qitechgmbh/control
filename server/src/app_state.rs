use crate::{
    ethercat::{
        config::{MAX_SUBDEVICES, PDI_LEN},
        device_identification::MachineDeviceIdentification,
    },
    socketio::room::Rooms,
};
use ethercat_hal::{actors::Actor, devices::Device};
use ethercrab::{subdevice_group::Op, MainDevice, SubDeviceGroup};
use socketioxide::SocketIo;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::RwLock;

pub struct AppState {
    pub socketio_rooms: RwLock<Rooms>,
    pub socketio: RwLock<Option<SocketIo>>,
    pub ethercat_setup: Arc<RwLock<Option<EthercatSetup>>>,
}

pub struct EthercatSetup {
    pub maindevice: MainDevice<'static>,
    pub group: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
    pub devices: Vec<Option<Arc<RwLock<dyn Device>>>>,
    pub device_groups: Vec<Vec<MachineDeviceIdentification>>,
    pub undetected_devices: Vec<MachineDeviceIdentification>,
    pub actors: Vec<Arc<RwLock<dyn Actor>>>,
    pub delays: Vec<Option<u32>>,
}

impl EthercatSetup {
    pub fn new(
        ethercat_master: MainDevice<'static>,
        ethercat_group: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
        ethercat_devices: Vec<Option<Arc<RwLock<dyn Device>>>>,
        ethercat_device_groups: Vec<Vec<MachineDeviceIdentification>>,
        ethercat_undetected_devices: Vec<MachineDeviceIdentification>,
        ethercat_actors: Vec<Arc<RwLock<dyn Actor>>>,
        ethercat_propagation_delays: Vec<Option<u32>>,
    ) -> Self {
        Self {
            maindevice: ethercat_master,
            group: ethercat_group,
            devices: ethercat_devices,
            device_groups: ethercat_device_groups,
            undetected_devices: ethercat_undetected_devices,
            actors: ethercat_actors,
            delays: ethercat_propagation_delays,
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
