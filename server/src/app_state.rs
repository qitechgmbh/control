use crate::ethercat::config::{MAX_SUBDEVICES, PDI_LEN};
use crate::rest::handlers::write_machine_device_identification::MachineDeviceInfoRequest;
use crate::socketio::main_namespace::MainNamespaceEvents;
use crate::socketio::main_namespace::machines_event::{MachineObj, MachinesEventBuilder};
use crate::socketio::namespaces::Namespaces;
use control_core::socketio::event::GenericEvent;
use ethercat_hal::devices::EthercatDevice;
use ethercrab::SubDeviceRef;
use ethercrab::{MainDevice, SubDeviceGroup, subdevice_group::Op};
use machines::machine_identification::{DeviceIdentification, MachineIdentificationUnique};
use machines::serial::registry::SERIAL_DEVICE_REGISTRY;
use machines::{Machine, MachineMessage};
use serde::{Deserialize, Serialize};
use smol::channel::{Receiver, Sender};
use smol::lock::{Mutex, RwLock};
use socketioxide::SocketIo;
use socketioxide::extract::SocketRef;
use std::collections::{HashMap, HashSet};
use std::fmt;
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

/*
Maybe better name needed ...
Essentially isntead of changing machines with locks, we send messages to our "HOT" thread wehere the machines are executed.
In there the Machines are built, destroyed, inspected etcetera.

Only Machines requiring RT capabilities (Mostly EtherCat machines) should be added
*/
pub enum HotThreadMessage {
    NoMsg,
    AddMachines(Vec<Box<dyn Machine>>),
    AddEtherCatSetup(EthercatSetup),
    WriteMachineDeviceInfo(MachineDeviceInfoRequest),
    DeleteMachine(MachineIdentificationUnique),
}

use crate::AsyncThreadMessage;

/*
    Instead of locking, etc only capture metadata on setup
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EtherCatDeviceMetaData {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub device_identification: DeviceIdentification,
}

impl EtherCatDeviceMetaData {
    pub fn from_subdevice(
        subdevice: &SubDeviceRef<'_, &ethercrab::SubDevice>,
        device_identification: DeviceIdentification,
    ) -> Self {
        Self {
            name: subdevice.name().to_string(),
            configured_address: subdevice.configured_address(),
            product_id: subdevice.identity().product_id,
            revision: subdevice.identity().revision,
            vendor_id: subdevice.identity().vendor_id,
            device_identification,
        }
    }
}

pub struct SharedState {
    pub socketio_setup: SocketioSetup,
    pub api_machines: Mutex<HashMap<MachineIdentificationUnique, Sender<MachineMessage>>>,
    pub current_machines_meta: Mutex<Vec<MachineObj>>,
    pub rt_machine_creation_channel: Sender<HotThreadMessage>,
    pub main_channel: Sender<AsyncThreadMessage>,
    pub ethercat_meta_data: RwLock<Vec<EtherCatDeviceMetaData>>,
}

impl fmt::Debug for EthercatSetup {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

use control_core::socketio::namespace::NamespaceCacheingLogic;

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

impl SharedState {
    pub async fn send_machines_event(&self) {
        let event = MachinesEventBuilder().build(self.current_machines_meta.lock().await.clone());
        let main_namespace = &mut self.socketio_setup.namespaces.write().await.main_namespace;
        main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
    }

    /// Removes a machine by its unique identifier
    pub async fn remove_machine(&self, machine_id: &MachineIdentificationUnique) {
        let mut current_machines = self.current_machines_meta.lock().await;
        // Retain only machines that do not match the given ID
        current_machines.retain(|m| &m.machine_identification_unique != machine_id);
        drop(current_machines);

        tracing::info!(
            "remove_machine {:?} {:?}",
            self.current_machines_meta,
            self.api_machines
        );
    }

    pub async fn add_machines_if_not_exists(&self, machines: Vec<MachineObj>) {
        let mut current_machines = self.current_machines_meta.lock().await;
        tracing::info!("add_machines_if_not_exists: {:?}", current_machines);
        // Track existing machine identifiers for quick lookup
        let existing_ids: HashSet<_> = current_machines
            .iter()
            .map(|m| m.machine_identification_unique.clone())
            .collect();

        for machine in machines {
            if !existing_ids.contains(&machine.machine_identification_unique) {
                current_machines.push(machine);
            }
        }
        drop(current_machines);

        self.send_machines_event().await;
    }

    pub async fn report_machine_error(
        &self,
        machine_identification_unique: MachineIdentificationUnique,
        error: String,
    ) {
        let mut current_machines = self.current_machines_meta.lock().await;

        for machine in current_machines.iter_mut() {
            if machine.machine_identification_unique == machine_identification_unique {
                machine.error = Some(error);
                return;
            }
        }

        current_machines.push(MachineObj {
            machine_identification_unique,
            error: Some(error),
        });
    }

    pub async fn add_machines(&self, machines: Vec<Box<dyn Machine>>) {
        let mut api_machines = self.api_machines.lock().await;
        for machine in machines.iter() {
            api_machines.insert(
                machine.get_machine_identification_unique(),
                machine.api_get_sender(),
            );
        }
        drop(api_machines);

        let objs = machines
            .iter()
            .map(|m| MachineObj {
                machine_identification_unique: m.get_machine_identification_unique(),
                error: None,
            })
            .collect();
        self.add_machines_if_not_exists(objs).await;

        self.rt_machine_creation_channel
            .send(HotThreadMessage::AddMachines(machines))
            .await
            .expect("Could not send to real-time channel");

        self.send_machines_event().await;
    }

    pub fn new(
        sender: Sender<HotThreadMessage>,
        main_async_channel: Sender<AsyncThreadMessage>,
    ) -> Self {
        let (socket_queue_tx, socket_queue_rx) = smol::channel::unbounded();
        Self {
            current_machines_meta: vec![].into(),
            ethercat_meta_data: vec![].into(),
            socketio_setup: SocketioSetup {
                socketio: RwLock::new(None),
                namespaces: RwLock::new(Namespaces::new(socket_queue_tx.clone())),
                socket_queue_tx,
                socket_queue_rx,
            },
            api_machines: Mutex::new(HashMap::new()),
            rt_machine_creation_channel: sender,
            main_channel: main_async_channel,
        }
    }
}
