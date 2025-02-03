use crate::{
    ethercat::config::{MAX_SUBDEVICES, PDI_LEN},
    ethercat_drivers::{actor::Actor, device::EthercatDevice},
    socketio::room::Rooms,
};
use ethercrab::{subdevice_group::Op, MainDevice, SubDeviceGroup};
use parking_lot::RwLock;
use socketioxide::SocketIo;
use std::sync::Arc;
use std::sync::LazyLock;

pub struct AppState {
    pub socketio_rooms: RwLock<Rooms>,
    pub socketio: RwLock<Option<SocketIo>>,
    pub ethercat_setup: Arc<RwLock<Option<EthercatSetup>>>,
}

pub struct EthercatSetup {
    pub maindevice: MainDevice<'static>,
    pub group: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
    pub devices: Vec<Option<Arc<RwLock<dyn EthercatDevice>>>>,
    pub actors: Vec<Arc<RwLock<dyn Actor>>>,
    pub delays: Vec<Option<u32>>,
}

impl EthercatSetup {
    pub fn new(
        ethercat_master: MainDevice<'static>,
        ethercat_group: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
        ethercat_devices: Vec<Option<Arc<RwLock<dyn EthercatDevice>>>>,
        ethercat_actors: Vec<Arc<RwLock<dyn Actor>>>,
        ethercat_propagation_delays: Vec<Option<u32>>,
    ) -> Self {
        Self {
            maindevice: ethercat_master,
            group: ethercat_group,
            devices: ethercat_devices,
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
