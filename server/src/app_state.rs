use crate::{
    ethercat::config::{MAX_SUBDEVICES, PDI_LEN},
    socketio::room::Rooms,
};
use ethercrab::{subdevice_group::Op, MainDevice, SubDeviceGroup};
use socketioxide::SocketIo;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::RwLock;
pub struct Ethercat {
    pub devices: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN>,
    pub master: MainDevice<'static>,
}

pub struct AppState {
    pub socketio_rooms: RwLock<Rooms>,
    pub socketio: RwLock<Option<SocketIo>>,
    pub ethercat_devices: RwLock<Option<SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>>>,
    pub ethercat_master: RwLock<Option<MainDevice<'static>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            socketio_rooms: RwLock::new(Rooms::new()),
            socketio: RwLock::new(None),
            ethercat_devices: RwLock::new(None),
            ethercat_master: RwLock::new(None),
        }
    }
}

pub static APP_STATE: LazyLock<Arc<AppState>> = LazyLock::new(|| Arc::new(AppState::new()));
