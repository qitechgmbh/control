use anyhow::{Error, Result};
use control_core::socketio::event::GenericEvent;
use control_core::socketio::namespace::{CacheableEvents, Namespace, NamespaceCacheingLogic};
use ethercat_hal::devices::{
    EthercatDevice, SubDeviceIdentityTuple, downcast_device, subdevice_identity_to_tuple,
};
use ethercat_hal::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use ethercrab::{SubDevice, SubDeviceRef};
use machine_identification::{
    DeviceHardwareIdentification, DeviceHardwareIdentificationEthercat, DeviceIdentification,
    DeviceIdentificationIdentified, MachineIdentificationUnique,
};
use serde::Serialize;
use smol::channel::{Receiver, Sender};
use socketioxide::extract::SocketRef;
use std::fmt::Debug;
use std::{any::Any, sync::Arc, time::Instant};
pub mod analog_input_test_machine;
pub mod aquapath1;
#[cfg(not(feature = "mock-machine"))]
pub mod buffer1;
pub mod extruder1;
pub mod extruder2;
pub mod ip20_test_machine;
pub mod laser;
pub mod machine_identification;
pub mod mock;
pub mod registry;
pub mod serial;
pub mod test_machine;
pub mod wago_power;
pub mod winder2;

pub const VENDOR_QITECH: u16 = 0x0001;
pub const MACHINE_WINDER_V1: u16 = 0x0002;
pub const MACHINE_EXTRUDER_V1: u16 = 0x0004;
pub const MACHINE_LASER_V1: u16 = 0x0006;
pub const MACHINE_MOCK: u16 = 0x0007;
#[cfg(not(feature = "mock-machine"))]
pub const MACHINE_BUFFER_V1: u16 = 0x0008;
pub const MACHINE_AQUAPATH_V1: u16 = 0x0009;
pub const MACHINE_WAGO_POWER_V1: u16 = 0x000A;
pub const MACHINE_EXTRUDER_V2: u16 = 0x0016;
pub const TEST_MACHINE: u16 = 0x0033;
pub const IP20_TEST_MACHINE: u16 = 0x0034;
pub const ANALOG_INPUT_TEST_MACHINE: u16 = 0x0035;

use serde_json::Value;
use smol::lock::RwLock;

#[derive(Serialize, Debug, Clone)]
pub struct MachineCrossConnectionState {
    machine_identification_unique: Option<MachineIdentificationUnique>,
    is_available: bool,
}

pub struct CrossConnection {
    pub src: MachineIdentificationUnique,
    pub dest: MachineIdentificationUnique,
}

pub enum AsyncThreadMessage {
    NoMsg,
    ConnectOneWayRequest(CrossConnection),
    DisconnectMachines(CrossConnection),
}

pub struct MachineNewParams<
    'maindevice,
    'subdevices,
    'device_identifications_identified,
    'ethercat_devices,
    'machine_new_hardware_etehrcat,
    'machine_new_hardware_serial,
    'machine_new_hardware,
> where
    'maindevice: 'machine_new_hardware,
    'subdevices: 'machine_new_hardware,
    'ethercat_devices: 'machine_new_hardware,
    'machine_new_hardware_etehrcat: 'machine_new_hardware,
{
    pub device_group: &'device_identifications_identified Vec<DeviceIdentificationIdentified>,
    pub hardware: &'machine_new_hardware MachineNewHardware<
        'maindevice,
        'subdevices,
        'ethercat_devices,
        'machine_new_hardware_etehrcat,
        'machine_new_hardware_serial,
    >,
    pub socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
    pub main_thread_channel: Option<Sender<AsyncThreadMessage>>,
    pub namespace: Option<Namespace>,
}

impl MachineNewParams<'_, '_, '_, '_, '_, '_, '_> {
    pub fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.device_group
            .first()
            .expect("device group must have at least one device")
            .device_machine_identification
            .machine_identification_unique
            .clone()
    }
}

pub enum MachineNewHardware<
    'maindevice,
    'subdevices,
    'ethercat_devices,
    'machine_new_hardware_etehrcat,
    'machine_new_hardware_serial,
> where
    'maindevice: 'machine_new_hardware_etehrcat,
    'subdevices: 'machine_new_hardware_etehrcat,
    'ethercat_devices: 'machine_new_hardware_etehrcat,
{
    Ethercat(
        &'machine_new_hardware_etehrcat MachineNewHardwareEthercat<
            'maindevice,
            'subdevices,
            'ethercat_devices,
        >,
    ),
    Serial(&'machine_new_hardware_serial MachineNewHardwareSerial),
}

pub struct MachineNewHardwareEthercat<'maindevice, 'subdevices, 'ethercat_devices> {
    pub subdevices:
        &'subdevices Vec<&'subdevices SubDeviceRef<'maindevice, &'subdevices SubDevice>>,
    pub ethercat_devices: &'ethercat_devices Vec<Arc<RwLock<dyn EthercatDevice>>>,
}

pub trait SerialDevice: Any + Send + Sync + SerialDeviceNew + Debug {}

pub trait SerialDeviceNew {
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Self>>), anyhow::Error>
    where
        Self: Sized;
}

pub trait SerialDeviceThread {
    fn start_thread() -> Result<(), anyhow::Error>;
}

pub struct SerialDeviceNewParams {
    pub path: String,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct SerialDeviceIdentification {
    pub vendor_id: u16,
    pub product_id: u16,
}

pub struct MachineNewHardwareSerial {
    pub device: Arc<RwLock<dyn SerialDevice>>,
}

// validates that all devices in the group have the same machine identification
pub fn validate_same_machine_identification_unique(
    identified_device_group: &Vec<DeviceIdentificationIdentified>,
) -> Result<(), Error> {
    let machine_identification_unique = &identified_device_group
        .first()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "[{}::validate_same_machine_identification] No devices in group",
                module_path!()
            )
        })?
        .device_machine_identification
        .machine_identification_unique;
    for device in identified_device_group.iter() {
        if device
            .device_machine_identification
            .machine_identification_unique
            != *machine_identification_unique
        {
            return Err(anyhow::anyhow!(
                "[{}::validate_same_machine_identification] Different machine identifications",
                module_path!()
            ));
        }
    }
    Ok(())
}

/// validates that every role is unique
pub fn validate_no_role_dublicates(
    identified_device_group: &Vec<DeviceIdentificationIdentified>,
) -> Result<(), Error> {
    let mut roles = vec![];
    for device in identified_device_group.iter() {
        if roles.contains(&device.device_machine_identification.role) {
            return Err(anyhow::anyhow!(
                "[{}::validate_no_role_dublicates] Role dublicate",
                module_path!(),
            ));
        }
        roles.push(device.device_machine_identification.role);
    }
    Ok(())
}

// Inside control_core::machines::new module:
pub fn get_device_identification_by_role(
    identified_device_group: &Vec<DeviceIdentificationIdentified>,
    role: u16,
) -> Result<&DeviceIdentificationIdentified, Error> {
    for device in identified_device_group.iter() {
        if device.device_machine_identification.role == role {
            return Ok(device);
        }
    }
    Err(anyhow::anyhow!(
        "[{}::get_device_identification_by_role] Role {} not found",
        module_path!(),
        role
    ))
}

pub fn get_device_by_index<'maindevice>(
    devices: &Vec<Arc<RwLock<dyn EthercatDevice>>>,
    subdevice_index: usize,
) -> Result<Arc<RwLock<dyn EthercatDevice>>, Error> {
    Ok(devices
        .get(subdevice_index)
        .ok_or(anyhow::anyhow!(
            "[{}::get_device_by_index] Index {} out of bounds for devices",
            module_path!(),
            subdevice_index
        ))?
        .clone())
}

pub fn get_subdevice_by_index<'subdevices, 'maindevice>(
    subdevices: &'subdevices Vec<&EthercrabSubDevicePreoperational<'maindevice>>,
    subdevice_index: usize,
) -> Result<&'subdevices EthercrabSubDevicePreoperational<'maindevice>, Error> {
    Ok(subdevices.get(subdevice_index).ok_or(anyhow::anyhow!(
        "Index {} out of bounds for subdevices",
        subdevice_index
    ))?)
}

pub fn get_ethercat_device_by_index<'maindevice>(
    ethercat_devices: &Vec<Arc<RwLock<dyn EthercatDevice>>>,
    subdevice_index: usize,
) -> Result<Arc<RwLock<dyn EthercatDevice>>, Error> {
    Ok(ethercat_devices
        .get(subdevice_index)
        .ok_or(anyhow::anyhow!(
            "[{}::get_ethercat_device_by_index] Index {} out of bounds for ethercat devices",
            module_path!(),
            subdevice_index
        ))?
        .clone())
}

pub trait MachineNewTrait: Machine {
    fn new(params: &MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct MachineConnection {
    pub ident: MachineIdentificationUnique,
    pub connection: Sender<MachineMessage>,
}

pub trait MachineAct {
    fn act_machine_message(&mut self, msg: MachineMessage);
    fn act(&mut self, now: Instant);
}

// generic MachineMessage allows us to implement actions
// to manage or mutate machines with simple messages sent to the Recv Channel of the given Machine,
// which the machine itself will handle to avoid locking
// This also allows for simplified "CrossConnections"
#[derive(Debug)]
pub enum MachineMessage {
    SubscribeNamespace(Namespace),
    UnsubscribeNamespace,
    HttpApiJsonRequest(serde_json::Value),
    ConnectToMachine(MachineConnection),
    DisconnectMachine(MachineConnection),
}

pub trait MachineApi {
    fn api_get_sender(&self) -> Sender<MachineMessage>;
    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error>;
    fn api_event_namespace(&mut self) -> Option<Namespace>;
}

pub trait Machine: MachineAct + MachineApi + Any + Debug + Send + Sync {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique;
    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>>;
}

pub trait AnyGetters: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

async fn get_device_ident<
    'maindevice,
    'subdevices,
    'device_identifications_identified,
    'ethercat_devices,
    'machine_new_hardware_etehrcat,
    'machine_new_hardware_serial,
    'machine_new_hardware,
>(
    params: &MachineNewParams<
        'maindevice,
        'subdevices,
        'device_identifications_identified,
        'ethercat_devices,
        'machine_new_hardware_etehrcat,
        'machine_new_hardware_serial,
        'machine_new_hardware,
    >,
    role: u16,
) -> Result<DeviceHardwareIdentificationEthercat, anyhow::Error> {
    let device_identification = get_device_identification_by_role(params.device_group, role)?;
    let device_hardware_identification_ethercat =
        match &device_identification.device_hardware_identification {
            DeviceHardwareIdentification::Ethercat(device_hardware_identification_ethercat) => {
                device_hardware_identification_ethercat
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/ExtruderV2::new] Device with role {} is not Ethercat",
                    module_path!(),
                    role
                ));
            }
        };
    return Ok(device_hardware_identification_ethercat.clone());
}

async fn get_ethercat_device<
    'maindevice,
    'subdevices,
    'device_identifications_identified,
    'ethercat_devices,
    'machine_new_hardware_etehrcat,
    'machine_new_hardware_serial,
    'machine_new_hardware,
    T,
>(
    hardware: &&MachineNewHardwareEthercat<'maindevice, 'subdevices, 'ethercat_devices>,
    params: &MachineNewParams<
        'maindevice,
        'subdevices,
        'device_identifications_identified,
        'ethercat_devices,
        'machine_new_hardware_etehrcat,
        'machine_new_hardware_serial,
        'machine_new_hardware,
    >,
    role: u16,
    expected_identities: Vec<SubDeviceIdentityTuple>,
) -> Result<
    (
        Arc<RwLock<T>>,
        &'subdevices SubDeviceRef<'subdevices, &'subdevices SubDevice>,
    ),
    anyhow::Error,
>
where
    T: 'static + Send + Sync + EthercatDevice,
{
    let device_hardware_identification_ethercat = get_device_ident(params, role).await?;
    let subdevice_index = device_hardware_identification_ethercat.subdevice_index;

    let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
    let subdevice_identity = subdevice.identity();

    let actual_identity = subdevice_identity_to_tuple(&subdevice_identity);

    let mut matched_any_identity = false;
    for identity in expected_identities.clone() {
        if actual_identity == identity {
            matched_any_identity = true;
        }
    }

    if !matched_any_identity {
        return Err(anyhow::anyhow!(
            "[{}::MachineNewTrait/ExtruderV2::new] Device identity mismatch: expected {:?}",
            module_path!(),
            expected_identities
        ));
    }

    let ethercat_device =
        get_ethercat_device_by_index(&hardware.ethercat_devices, subdevice_index)?;
    let device = downcast_device::<T>(ethercat_device).await?;

    {
        let mut device_guard = device.write().await;
        device_guard.set_used(true);
    }

    Ok((device, subdevice))
}

#[derive(Debug)]
pub struct MachineChannel {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    main_sender: Option<Sender<AsyncThreadMessage>>,
    namespace: Option<Namespace>,
}

impl MachineChannel {
    pub fn new(machine_identification_unique: MachineIdentificationUnique) -> Self {
        let (sender, receiver) = smol::channel::unbounded();

        Self {
            api_sender: sender,
            api_receiver: receiver,
            machine_identification_unique,
            main_sender: None,
            namespace: None,
        }
    }
}

impl<E> NamespaceCacheingLogic<E> for MachineChannel
where
    E: CacheableEvents<E>,
{
    fn emit(&mut self, events: E) {
        let event = Arc::new(events.event_value());
        let cache_fn = events.event_cache_fn();

        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &cache_fn);
        }
    }
}

pub trait MachineWithChannel: Send + Debug + Sync {
    fn get_machine_channel(&self) -> &MachineChannel;
    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel;

    fn on_namespace(&mut self) {}

    fn update(&mut self, now: std::time::Instant) -> Result<()>;
    fn mutate(&mut self, value: Value) -> Result<()>;
}

impl<C> MachineApi for C
where
    C: MachineWithChannel,
{
    fn api_get_sender(&self) -> Sender<MachineMessage> {
        self.get_machine_channel().api_sender.clone()
    }

    fn api_mutate(&mut self, value: Value) -> Result<()> {
        let res = self.mutate(value);

        if let Err(ref e) = res {
            tracing::error!("Machine errored while mutating: {}, ", e);
        }

        res
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.get_machine_channel().namespace.clone()
    }
}

impl<C> MachineAct for C
where
    C: MachineWithChannel,
{
    fn act(&mut self, now: Instant) {
        while let Ok(msg) = self.get_machine_channel_mut().api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        if let Err(e) = self.update(now) {
            tracing::error!("Machine errored while updating: {}, ", e);
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        let channel = self.get_machine_channel_mut();

        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                channel.namespace = Some(namespace);
                self.on_namespace();
            }
            MachineMessage::UnsubscribeNamespace => {
                channel.namespace = None;
            }
            MachineMessage::HttpApiJsonRequest(value) => {
                let _ = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection) => {
                todo!();
            }
            MachineMessage::DisconnectMachine(_machine_connection) => {
                todo!();
            }
        }
    }
}

impl<C> Machine for C
where
    C: MachineWithChannel + 'static,
{
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.get_machine_channel()
            .machine_identification_unique
            .clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.get_machine_channel().main_sender.clone()
    }
}
