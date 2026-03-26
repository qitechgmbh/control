use anyhow::{Result};


use qitech_lib::machines::Machine;
use serde::Serialize;
use smol::channel::{Sender};
use control_core::socketio::namespace::Namespace;
pub mod minimal_machines;

/*pub mod aquapath1;
#[cfg(not(feature = "mock-machine"))]
pub mod buffer1;
pub mod extruder1;
pub mod extruder2;
pub mod laser;
pub mod machine_identification;
pub mod minimal_machines;
pub mod registry;
pub mod serial;
pub mod wago_power;*/
/*pub mod wago_serial_machine;*/
/*pub mod winder2;*/

mod machine_data;
pub mod machine_identification;
pub use machine_data::MachineData;

pub const VENDOR_QITECH: u16 = 0x0001;
pub const MACHINE_WINDER_V1: u16 = 0x0002;
pub const MACHINE_EXTRUDER_V1: u16 = 0x0004;
pub const MACHINE_LASER_V1: u16 = 0x0006;
pub const MACHINE_MOCK: u16 = 0x0007;
pub const MACHINE_BUFFER_V1: u16 = 0x0008;
pub const MACHINE_AQUAPATH_V1: u16 = 0x0009;
pub const MACHINE_WAGO_POWER_V1: u16 = 0x000A;
pub const MACHINE_EXTRUDER_V2: u16 = 0x0016;
pub const TEST_MACHINE: u16 = 0x0033;
pub const IP20_TEST_MACHINE: u16 = 0x0034;
pub const ANALOG_INPUT_TEST_MACHINE: u16 = 0x0035;
pub const WAGO_AI_TEST_MACHINE: u16 = 0x0036;
pub const DIGITAL_INPUT_TEST_MACHINE: u16 = 0x0040;
pub const WAGO_8CH_IO_TEST_MACHINE: u16 = 0x0041;
pub const WAGO_750_430_DI_MACHINE: u16 = 0x0043;
pub const WAGO_750_553_MACHINE: u16 = 0x0044;
pub const TEST_MACHINE_STEPPER: u16 = 0x0037;
pub const MOTOR_TEST_MACHINE: u16 = 0x0011;
pub const WAGO_DO_TEST_MACHINE: u16 = 0x000E;
pub const WAGO_750_501_TEST_MACHINE: u16 = 0x0042;


#[derive(Serialize, Debug, Clone)]
pub struct MachineValues {
    pub state: serde_json::Value,
    pub live_values: serde_json::Value,
}

pub enum MachineMessage {
    SubscribeNamespace(Namespace),
    UnsubscribeNamespace,
    HttpApiJsonRequest(serde_json::Value),
    RequestValues(Sender<MachineValues>),
}

pub trait MachineApi  {
    fn act_machine_message(&mut self, msg: MachineMessage);
    fn api_get_sender(&self) -> Sender<MachineMessage>;
    fn api_mutate(&mut self, value: serde_json::Value) -> Result<(), anyhow::Error>;
    fn api_event_namespace(&mut self) -> Option<Namespace>;
}

// Generic Machine Init For us
pub trait MachineNew {

}

pub trait QiTechMachine : Machine + MachineApi {}

/*
use serde_json::Value;
use smol::lock::RwLock;

pub struct MachineSubscriptionRequest {
    pub subscriber: MachineIdentificationUnique,
    pub publisher: MachineIdentificationUnique,
}

pub enum AsyncThreadMessage {
    NoMsg,
    SubscribeToMachine(MachineSubscriptionRequest),
    UnsubscribeFromMachine(MachineSubscriptionRequest),
}

pub struct MachineNewParams
{
    pub device_group: Vec<DeviceIdentificationIdentified>,
    pub hardware: MachineNewHardware,
    pub socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
    pub main_thread_channel: Option<Sender<AsyncThreadMessage>>,
    pub ethercat_thread_channel: Option<EtherCATThreadChannel>,
    pub namespace: Option<Namespace>,
}

impl MachineNewParams {
    pub fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.device_group
            .first()
            .expect("device group must have at least one device")
            .device_machine_identification
            .machine_identification_unique
            .clone()
    }
}

pub enum MachineNewHardware
{
    Ethercat(
        MachineNewHardwareEthercat,
    ),
    Serial(MachineNewHardwareSerial),
}

pub struct MachineNewHardwareEthercat {
    pub ethercat_devices: Vec<Arc<Box<dyn EthercatDevice>>>,
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
pub fn validate_no_role_duplicates(
    identified_device_group: &Vec<DeviceIdentificationIdentified>,
) -> Result<(), Error> {
    let mut roles = vec![];
    for device in identified_device_group.iter() {
        if roles.contains(&device.device_machine_identification.role) {
            return Err(anyhow::anyhow!(
                "[{}::validate_no_role_duplicates] Role duplicate",
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
    fn new(params: &MachineNewParams) -> Result<Self, anyhow::Error>
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

#[derive(Serialize, Debug, Clone)]
pub struct MachineValues {
    pub state: serde_json::Value,
    pub live_values: serde_json::Value,
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
    RequestValues(Sender<MachineValues>),
}

pub trait MachineApi {
    fn api_get_sender(&self) -> Sender<MachineMessage>;
    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error>;
    fn api_event_namespace(&mut self) -> Option<Namespace>;
}
/*
pub trait Machine: MachineAct + MachineApi + Any + Debug {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique;
    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>>;
    fn mutation_counter(&self) -> u64 {
        0
    }
    fn update_machine_data(
        &self,
        data: &mut MachineData,
        refresh_state: bool,
        refresh_live_values: bool,
    ) {
        _ = data;
        _ = refresh_state;
        _ = refresh_live_values;
    }

    fn receive_machines_data(&mut self, data: &MachineData) {
        _ = data;
    }

    fn subscribed_to_machine(&mut self, uid: MachineIdentificationUnique) {
        _ = uid;
    }

    fn unsubscribed_from_machine(&mut self, uid: MachineIdentificationUnique) {
        _ = uid;
    }
}
*/
pub trait AnyGetters: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

fn get_device_ident(
    params: &MachineNewParams,
    role: u16,
) -> Result<DeviceHardwareIdentificationEthercat, anyhow::Error> {
    let device_identification = get_device_identification_by_role(&params.device_group, role)?;
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

fn get_ethercat_device<T>(
    hardware: &Vec<Box<dyn EthercatDevice>>,
    role: u16,
    expected_identities: Vec<SubDeviceIdentityTuple>,
) -> Result<Box<T>,anyhow::Error>
where
    T: EthercatDevice
{
/**/
    todo!();
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
    type State: serde::Serialize;
    type LiveValues: serde::Serialize;

    fn get_machine_channel(&self) -> &MachineChannel;
    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel;

    fn on_namespace(&mut self) {}

    fn update(&mut self, now: std::time::Instant) -> Result<()>;
    fn mutate(&mut self, value: Value) -> Result<()>;

    fn get_state(&self) -> Self::State;
    fn get_live_values(&self) -> Option<Self::LiveValues> {
        None
    }
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
            MachineMessage::RequestValues(sender) => {
                sender
                    .send_blocking(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
                sender.close();
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
*/
