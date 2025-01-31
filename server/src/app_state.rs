use crate::{
    ethercat::config::{MAX_SUBDEVICES, PDI_LEN},
    ethercat_drivers::{actor::Actor, device::Device},
    socketio::room::Rooms,
};
use ethercrab::{subdevice_group::Op, MainDevice, SubDeviceGroup};
use socketioxide::SocketIo;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::RwLock;

pub struct AppState {
    pub socketio_rooms: RwLock<Rooms>,
    pub socketio: RwLock<Option<SocketIo>>,
    pub ethercat_master: Arc<RwLock<Option<MainDevice<'static>>>>,
    pub ethercat_group: Arc<RwLock<Option<SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>>>>,
    pub ethercat_devices: Arc<RwLock<Option<Vec<Option<Arc<RwLock<dyn Device>>>>>>>,
    pub ethercat_actors: Arc<RwLock<Option<Vec<Arc<RwLock<dyn Actor>>>>>>,
    pub ethercat_propagation_delays: Arc<RwLock<Option<Vec<u32>>>>,
}

pub struct EthercatSetup {}

impl AppState {
    pub fn new() -> Self {
        Self {
            socketio_rooms: RwLock::new(Rooms::new()),
            socketio: RwLock::new(None),
            ethercat_master: Arc::new(RwLock::new(None)),
            ethercat_group: Arc::new(RwLock::new(None)),
            ethercat_devices: Arc::new(RwLock::new(None)),
            ethercat_actors: Arc::new(RwLock::new(None)),
            ethercat_propagation_delays: Arc::new(RwLock::new(None)),
        }
    }
}

pub static APP_STATE: LazyLock<Arc<AppState>> = LazyLock::new(|| Arc::new(AppState::new()));
