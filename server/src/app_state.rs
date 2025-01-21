use std::sync::Arc;

use ethercrab::{MainDevice, SubDeviceGroup};
use std::sync::LazyLock;
use tokio::sync::RwLock;

use crate::ethercat::config::{MAX_SUBDEVICES, PDI_LEN};

pub struct AppState {
    pub group: Arc<RwLock<Option<SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN>>>>,
    pub maindevice: Arc<RwLock<Option<MainDevice<'static>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            group: Arc::new(RwLock::new(None)),
            maindevice: Arc::new(RwLock::new(None)),
        }
    }
}
pub static APP_STATE: LazyLock<Arc<AppState>> = LazyLock::new(|| Arc::new(AppState::new()));
