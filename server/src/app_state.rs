use crate::ethercat::config::{MAX_SUBDEVICES, PDI_LEN};
use crate::serial::dre::Dre;
use crate::serial::serial_detection::SerialDetection;
use crate::socketio::namespaces::Namespaces;
use control_core::machines::Machine;
use control_core::machines::identification::{DeviceIdentification, MachineIdentificationUnique};
use control_core::machines::manager::MachineManager;
use control_core::serial::registry::SerialRegistry;
use control_core::serial::{ProductConfig, Serial};
use ethercat_hal::devices::EthercatDevice;
use ethercrab::{MainDevice, SubDeviceGroup, subdevice_group::Op};
use smol::lock::RwLock;
use socketioxide::SocketIo;
use std::collections::HashMap;
use std::sync::Arc;

pub struct SocketioSetup {
    pub socketio: RwLock<Option<SocketIo>>,
    pub namespaces: RwLock<Namespaces>,
}

pub struct AppState {
    pub socketio_setup: SocketioSetup,
    pub ethercat_setup: Arc<RwLock<Option<EthercatSetup>>>,
    pub serial_setup: Arc<RwLock<Option<SerialSetup>>>,
    pub machines: RwLock<MachineManager>,
}

pub type Machines =
    HashMap<MachineIdentificationUnique, Result<RwLock<dyn Machine>, anyhow::Error>>;

pub struct EthercatSetup {
    /// All Ethercat devices
    /// Device-Specific interface for all devices
    /// Same length and order as SubDevices inside `group` (index = subdevice_index)
    pub devices: Vec<(DeviceIdentification, Arc<RwLock<dyn EthercatDevice>>)>,
    /// All Ethercat devices
    /// Generic interface for all devices
    /// Needed to interface with the devices on an Ethercat level
    pub group: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
    /// The Ethercat main device
    /// Needed to interface with the devices
    pub maindevice: MainDevice<'static>,
}

impl EthercatSetup {
    pub fn new(
        devices: Vec<(DeviceIdentification, Arc<RwLock<dyn EthercatDevice>>)>,
        group: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
        maindevice: MainDevice<'static>,
    ) -> Self {
        Self {
            devices,
            group,
            maindevice,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            socketio_setup: SocketioSetup {
                socketio: RwLock::new(None),
                namespaces: RwLock::new(Namespaces::new()),
            },
            ethercat_setup: Arc::new(RwLock::new(None)),
            serial_setup: Arc::new(RwLock::new(None)),
            machines: RwLock::new(MachineManager::new()),
        }
    }
}

pub struct SerialSetup{
    pub connected_serial: HashMap<String, Result<Arc<RwLock<dyn Serial>>, anyhow::Error>>,
    pub serial_detection: Arc<RwLock<SerialDetection>>,
}

impl SerialSetup {
    pub async fn new() -> Self {
        let registry =  SerialRegistry::new();
        let sd = Arc::new(RwLock::new(SerialDetection::new(registry.clone())));
        Self {
            connected_serial: sd.read().await.connected_serial_usb.iter()
                .map(|(k, v)| (k.clone(), v.as_ref().map(|arc| Arc::clone(arc)).map_err(|e| anyhow::Error::msg(e.to_string()))))
                .collect(),
            serial_detection: sd.clone(),
        }
    }
    pub async fn register_config(&mut self, pc: ProductConfig) {
        self.serial_detection.write().await.sr.register::<Dre>(pc.clone());
    }
}