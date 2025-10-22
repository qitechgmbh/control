use crate::ethercat::config::{MAX_SUBDEVICES, PDI_LEN};
use crate::performance_metrics::EthercatPerformanceMetrics;
use crate::serial::registry::SERIAL_DEVICE_REGISTRY;
use crate::socketio::main_namespace::machines_event::MachineObj;
use crate::socketio::namespaces::Namespaces;
use control_core::machines::Machine;
use control_core::machines::identification::{DeviceIdentification, MachineIdentificationUnique};
use control_core::machines::manager::MachineManager;
use control_core::socketio::event::GenericEvent;
use ethercat_hal::devices::EthercatDevice;
use ethercrab::{MainDevice, SubDeviceGroup, subdevice_group::Op};
use smol::channel::{Receiver, Sender};
use smol::lock::RwLock;
use socketioxide::SocketIo;
use socketioxide::extract::SocketRef;
use std::collections::HashMap;
use std::sync::Arc;

pub struct SocketioSetup {
    pub socketio: RwLock<Option<SocketIo>>,
    pub namespaces: RwLock<Namespaces>,
    pub socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
    pub socket_queue_rx: Receiver<(SocketRef, Arc<GenericEvent>)>,
}

pub struct SerialSetup {
    pub serial_registry: &'static SERIAL_DEVICE_REGISTRY,
}

pub struct AppState {
    pub socketio_setup: SocketioSetup,
    pub ethercat_setup: Arc<RwLock<Option<EthercatSetup>>>,
    pub serial_setup: Arc<RwLock<SerialSetup>>,
    pub machines: Arc<RwLock<MachineManager>>,
    pub performance_metrics: Arc<RwLock<EthercatPerformanceMetrics>>,
}

pub type Machines =
    HashMap<MachineIdentificationUnique, Result<Arc<RwLock<dyn Machine>>, anyhow::Error>>;

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

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (socket_queue_tx, socket_queue_rx) = smol::channel::unbounded();
        Self {
            socketio_setup: SocketioSetup {
                socketio: RwLock::new(None),
                namespaces: RwLock::new(Namespaces::new(socket_queue_tx.clone())),
                socket_queue_tx,
                socket_queue_rx,
            },
            ethercat_setup: Arc::new(RwLock::new(None)),
            serial_setup: Arc::new(RwLock::new(SerialSetup {
                serial_registry: &SERIAL_DEVICE_REGISTRY,
            })),
            machines: Arc::new(RwLock::new(MachineManager::new())),
            performance_metrics: Arc::new(RwLock::new(EthercatPerformanceMetrics::new())),
        }
    }

    pub fn get_machine_objs(&self) -> Vec<MachineObj> {
        let machines = self.machines.read_blocking();
        machines
            .iter()
            .map(|machine| {
                let error = {
                    let slot = machine.1.lock_blocking();
                    slot.machine_connection.to_error().map(|e| e.to_string())
                };

                MachineObj {
                    machine_identification_unique: machine.0.clone(),
                    error,
                }
            })
            .collect()
    }
}
