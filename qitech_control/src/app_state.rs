use crate::apis::socketio::{
    main_namespace::{
        MainNamespaceEvents,
        ethercat_devices_event::{
            EcatState, EtherCatDeviceMetaData, EthercatDevicesEvent, EthercatSetupDone,
        },
        ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent,
        machines_event::{MachineObj, MachinesEventBuilder},
    },
    namespaces::Namespaces,
};
use anyhow::bail;
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::NamespaceCacheingLogic,
};
use machine_implementations::{
    Hardware, IdentifiedEthercat, IdentifiedModbus, MachineHardware, MachineMessage, QiTechMachine,
    dryer::{DryerMachine, device::DryerDevice},
    dryer_smart::DryerSmartMachine,
    laser::LaserMachine,
    machine_identification::{
        DeviceHardwareIdentificationEthercat, DeviceIdentification, DeviceMachineIdentification,
        QiTechMachineIdentificationUnique,
    },
};
use qitech_lib::{
    ethercat_hal::{
        Consumer, EtherCATControl, EtherCATThreadChannel, MetaSubdevice, Producer,
        devices::EthercatDevice, machine_ident_read::MachineDeviceInfo,
    },
    machines::{MachineDataRegistry, MachineIdentification, MachineIdentificationUnique},
    modbus::{ModbusDevice, devices::qitech_laser::LaserDevice},
};
use socketioxide::{SocketIo, extract::SocketRef};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, OnceLock},
};
use tokio::{
    runtime::Runtime,
    sync::{
        RwLock,
        mpsc::{Receiver, Sender},
    },
};

static RUNTIME: OnceLock<Runtime> = OnceLock::new();
pub fn get_async_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio Runtime")
    })
}

pub struct SocketioSetup {
    pub socketio: RwLock<Option<SocketIo>>,
    pub namespaces: RwLock<Namespaces>,
    pub socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
    pub socket_queue_rx: RwLock<Receiver<(SocketRef, Arc<GenericEvent>)>>,
}

/*
    This struct is only written in the main machine loop or during initialization,
    Otherwise it is simply read.
    Except socketio
*/
pub struct SharedAppState {
    pub machines: RwLock<Vec<MachineObj>>,
    pub machines_with_channel:
        RwLock<HashMap<QiTechMachineIdentificationUnique, Sender<MachineMessage>>>,
    pub ethercat_meta_datas: RwLock<Vec<EtherCatDeviceMetaData>>,
    pub socketio_setup: SocketioSetup,
    pub ethercat_thread_channel: Option<EtherCATThreadChannel>,
}

impl SharedAppState {
    pub fn fill_ethercat_metadata<C: Consumer, P: Producer>(
        &self,
        controller: &EtherCATControl<C, P>,
        infos: Vec<MachineDeviceInfo>,
    ) -> Result<(), anyhow::Error> {
        let mut guard = self.ethercat_meta_datas.try_write()?;
        let subdevices = controller.app_handle.try_get_subdevices_vec_sync()?;
        for dev in subdevices {
            let device_machine_identification = infos
                .iter()
                .find(|info| info.device_address == dev.device_address)
                .map(|info| DeviceMachineIdentification::from(*info));

            guard.push(
                EtherCatDeviceMetaData {
                    configured_address: dev.device_address,
                    name: dev.get_name()?,
                    vendor_id: dev.vendor,
                    product_id: dev.product_id,
                    revision: dev.revision,
                    device_identification: DeviceIdentification{
                            device_machine_identification: device_machine_identification,
                            device_hardware_identification:
                                machine_implementations::machine_identification::DeviceHardwareIdentification::Ethercat(DeviceHardwareIdentificationEthercat{ subdevice_index: dev.device_address as usize })
                    }
            });
        }
        drop(guard);
        Ok(())
    }

    /// Emit an EtherCAT interface discovery event synchronously (non-async).
    /// Uses `try_write()` so it can be called from a synchronous context during
    /// startup, before the Tokio runtime and Socket.io server are fully running.
    /// Events are queued and delivered when the Socket.io queue consumer starts.
    pub fn emit_ethercat_interface_discovery(
        &self,
        event: EthercatInterfaceDiscoveryEvent,
    ) -> Result<(), anyhow::Error> {
        let built = event.build();
        let mut guard = self.socketio_setup.namespaces.try_write()?;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(built));
        drop(guard);
        Ok(())
    }

    pub async fn send_ethercat_setup_init(&self) {
        let event = Event::new(
            "EthercatDevicesEvent",
            EthercatDevicesEvent::Initializing(true),
        );
        let mut guard = self.socketio_setup.namespaces.write().await;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
        drop(guard);
    }

    pub async fn send_ethercat_setup_done(&self) {
        let event = Event::new(
            "EthercatDevicesEvent",
            EthercatDevicesEvent::Done(EthercatSetupDone {
                devices: self.ethercat_meta_datas.read().await.clone(),
            }),
        );
        let mut guard = self.socketio_setup.namespaces.write().await;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
        drop(guard);
    }

    pub async fn send_machines_event(&self) -> Result<(), anyhow::Error> {
        let event = MachinesEventBuilder().build(self.get_machines_meta().await);
        let mut guard = self.socketio_setup.namespaces.write().await;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
        drop(guard);
        Ok(())
    }

    pub async fn send_ethercat_state(&self, ecat_state: EcatState) {
        let event = Event::new(
            "EthercatStateEvent",
            EthercatDevicesEvent::State(ecat_state.into()),
        );
        let mut guard = self.socketio_setup.namespaces.write().await;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
        drop(guard);
    }

    pub async fn get_machines_meta(&self) -> Vec<MachineObj> {
        self.machines.read().await.clone()
    }

    pub async fn message_machine(
        &self,
        machine_identification_unique: &QiTechMachineIdentificationUnique,
        message: MachineMessage,
    ) -> Result<(), anyhow::Error> {
        let guard = self.machines_with_channel.read().await;
        let sender = guard.get(machine_identification_unique);
        if let Some(sender) = sender {
            sender.send(message).await?;
        }
        drop(guard);
        // why does a macro for return Err() exist bro ...
        bail!("Unknown machine!")
    }

    pub fn add_machine_sync(
        &self,
        ident: QiTechMachineIdentificationUnique,
        err: Option<String>,
        sender: Option<Sender<MachineMessage>>,
    ) -> Result<(), anyhow::Error> {
        let mut guard = self.machines.try_write()?;
        let machine_obj = MachineObj {
            machine_identification_unique: ident,
            error: err,
        };
        guard.push(machine_obj);
        drop(guard);

        match sender {
            Some(sender) => {
                let mut guard = self.machines_with_channel.try_write()?;
                guard.insert(ident, sender);
                drop(guard);
            }
            None => {}
        };

        Ok(())
    }

    pub async fn add_machine(
        &self,
        ident: QiTechMachineIdentificationUnique,
        err: Option<String>,
        sender: Sender<MachineMessage>,
    ) {
        let mut guard = self.machines.write().await;
        let machine_obj = MachineObj {
            machine_identification_unique: ident,
            error: err,
        };
        guard.push(machine_obj);
        drop(guard);

        let mut guard = self.machines_with_channel.write().await;
        guard.insert(ident, sender);
        drop(guard);
    }

    pub fn new() -> Self {
        let (socket_queue_tx, socket_queue_rx) = tokio::sync::mpsc::channel(64);
        Self {
            machines: RwLock::new(vec![]),
            machines_with_channel: RwLock::new(HashMap::new()),
            socketio_setup: SocketioSetup {
                socketio: RwLock::new(None),
                namespaces: RwLock::new(Namespaces::new(socket_queue_tx.clone())),
                socket_queue_tx,
                socket_queue_rx: RwLock::new(socket_queue_rx),
            },
            ethercat_meta_datas: RwLock::new(vec![]),
            ethercat_thread_channel: None,
        }
    }
}

pub struct MainState {
    pub subdevices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)>,
    pub hardware: HashMap<MachineIdentificationUnique, MachineHardware>,
    pub machines: Vec<Box<dyn QiTechMachine>>,
    pub machine_errors: HashMap<MachineIdentificationUnique, String>,
    pub machine_data_reg: MachineDataRegistry,
}

impl MainState {
    pub fn new() -> Self {
        let machines = vec![];
        let machine_data_reg = MachineDataRegistry {
            storage: HashMap::new(),
        };
        MainState {
            machines,
            machine_data_reg,
            subdevices: vec![],
            hardware: HashMap::new(),
            machine_errors: HashMap::new(),
        }
    }

    pub fn generate_machine_hardware_from_serial(
        &mut self,
        path: &str,
    ) -> Result<(), anyhow::Error> {
        let laser_device: Rc<RefCell<LaserDevice>> =
            Rc::new(RefCell::new(LaserDevice::new(path.to_owned(), 1, None)?));
        let id_modbus: IdentifiedModbus = IdentifiedModbus { hw: laser_device };
        let ident = MachineIdentificationUnique {
            machine_ident: LaserMachine::MACHINE_IDENTIFICATION,
            serial: 1,
        };
        let mut hw = MachineHardware {
            hw: vec![],
            identification: ident,
            ethercat_interface: None,
        };
        hw.hw.push(Hardware::Modbus(id_modbus));
        self.hardware.insert(ident, hw);
        Ok(())
    }

    pub fn generate_machine_hardware_from_dryer_serial(
        &mut self,
        path: &str,
    ) -> Result<(), anyhow::Error> {
        let dryer_device = DryerDevice::new(path.to_owned(), 1, None)?;
        // The V1/Smart variant is probed once at construction (see DryerDevice::new); pick
        // the matching machine identification before the machine itself gets built.
        let machine_ident = if dryer_device.is_smart {
            DryerSmartMachine::MACHINE_IDENTIFICATION
        } else {
            DryerMachine::MACHINE_IDENTIFICATION
        };
        let dryer_device: Rc<RefCell<DryerDevice>> = Rc::new(RefCell::new(dryer_device));
        let id_modbus: IdentifiedModbus = IdentifiedModbus { hw: dryer_device };
        let ident = MachineIdentificationUnique {
            machine_ident,
            serial: 1,
        };
        let mut hw = MachineHardware {
            hw: vec![],
            identification: ident,
            ethercat_interface: None,
        };
        hw.hw.push(Hardware::Modbus(id_modbus));
        self.hardware.insert(ident, hw);
        Ok(())
    }

    pub fn generate_machine_hardware_from_ethercat(
        &mut self,
        device_infos: &Vec<MachineDeviceInfo>,
        mapped_ecat_devices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)>,
        ethercat_channel: EtherCATThreadChannel,
    ) {
        // If an info points to the same address as the actuall device, we assume they should be
        // linked together. This is the only value from the MetaSubdevice, we can use here.
        let mut combined_list: Vec<(
            MachineDeviceInfo,
            MetaSubdevice,
            Rc<RefCell<dyn EthercatDevice>>,
        )> = device_infos
            .into_iter()
            .filter_map(|info| {
                let address = info.device_address;
                let f = mapped_ecat_devices
                    .iter()
                    .find(|f| f.0.device_address == address)
                    .unwrap();
                Some((info.clone(), f.0, f.1.clone())) // This is a 3-tuple
            })
            .collect();

        combined_list.sort_by_key(|f| (f.0.machine_id, f.0.machine_serial));
        for (dev_info, _, eth) in combined_list.drain(0..combined_list.len()) {
            // Here we try to get the MachineIdentificationUnique
            let identification = MachineIdentificationUnique {
                machine_ident: MachineIdentification {
                    vendor: dev_info.machine_vendor,
                    machine: dev_info.machine_id,
                },
                serial: dev_info.machine_serial as u32,
            };

            // If this machine has no hardware assigned yet, create empty list
            if self.hardware.get(&identification).is_none() {
                self.hardware.insert(
                    identification,
                    MachineHardware {
                        hw: vec![],
                        identification,
                        ethercat_interface: Some(ethercat_channel.clone()),
                    },
                );
            }

            // Add this device to the machine's hardware
            let ethercat_hw = Hardware::Ethercat(IdentifiedEthercat {
                hw: eth,
                ident: dev_info,
            });
            let hw = self.hardware.get_mut(&identification).unwrap();
            hw.hw.push(ethercat_hw);
        }
    }
}
