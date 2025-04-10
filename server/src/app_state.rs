use crate::ethercat::config::{MAX_SUBDEVICES, PDI_LEN};
use crate::socketio::namespaces::Namespaces;
use control_core::actors::Actor;
use control_core::identification::{MachineDeviceIdentification, MachineIdentificationUnique};
use control_core::machines::Machine;
use ethercat_hal::devices::Device;
use ethercrab::{subdevice_group::Op, MainDevice, SubDeviceGroup};
use smol::lock::RwLock;
use socketioxide::SocketIo;
use std::sync::Arc;
use std::{collections::HashMap, sync::LazyLock};

pub struct SocketioSetup {
    pub socketio: RwLock<Option<SocketIo>>,
    pub namespaces: RwLock<Namespaces>,
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
                namespaces: RwLock::new(Namespaces::new()),
            },
            ethercat_setup: Arc::new(RwLock::new(None)),
        }
    }
}

pub static APP_STATE: LazyLock<Arc<AppState>> = LazyLock::new(|| Arc::new(AppState::new()));
