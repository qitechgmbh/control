use crate::apis::socketio::{
    main_namespace::{
        MainNamespaceEvents,
        ethercat_devices_event::{EtherCatDeviceMetaData, EthercatDevicesEvent, EthercatSetupDone},
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
    Hardware, IdentifiedEthercat, IdentifiedModbus, MachineHardware, MachineMessage, QiTechMachine, laser::LaserMachine, machine_identification::{
        DeviceHardwareIdentificationEthercat, DeviceIdentification, DeviceMachineIdentification,
        QiTechMachineIdentificationUnique,
    }
};
use qitech_lib::{ethercat_hal::{EtherCATThreadChannel, MetaSubdevice, StandardEtherCATController, devices::EthercatDevice, machine_ident_read::MachineDeviceInfo}, machines::{MachineDataRegistry, MachineIdentification, MachineIdentificationUnique}, modbus::{devices::qitech_laser::LaserDevice, managers::{ExampleDeviceManager, example_manager::ExampleScheduler}}};
use socketioxide::{SocketIo, extract::SocketRef};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::{Arc, OnceLock}};
use tokio::{runtime::Runtime, sync::{
    RwLock,
    mpsc::{Receiver, Sender},
}};

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
}

impl SharedAppState {
        pub fn fill_ethercat_metadata(
        &self,
        controller: Arc<StandardEtherCATController>,
        infos: Vec<MachineDeviceInfo>,
    ) -> Result<(), anyhow::Error> {
        let mut guard = self.ethercat_meta_datas.try_write()?;
        for i in 0..controller.subdevice_count {
            let dev = controller.subdevices[i];
            let device_machine_identification = infos.iter()
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
        }
    }
}

pub struct MainState {
    pub subdevices: Vec<Rc<RefCell<dyn EthercatDevice>>>,
    pub hardware: HashMap<MachineIdentificationUnique, MachineHardware>,
    pub machines: Vec<Box<dyn QiTechMachine>>,
    pub machine_errors: HashMap<MachineIdentificationUnique, String>,
    pub machine_data_reg: MachineDataRegistry,
    pub modbus_mgrs: Vec<Rc<RefCell<ExampleDeviceManager>>>
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
            modbus_mgrs: vec![],
            hardware: HashMap::new(),
            machine_errors: HashMap::new(),
        }
    }

    pub fn generate_machine_hardware_from_serial(
        &mut self, mgr: Rc<RefCell<ExampleDeviceManager>>,
    ){
        let laser_device: Rc<RefCell<LaserDevice<ExampleScheduler>>> = ExampleDeviceManager::register_device(mgr.clone(), 1);
        let id_modbus : IdentifiedModbus = IdentifiedModbus { hw: laser_device,manager:mgr.clone() };
        let ident = MachineIdentificationUnique { machine_ident: LaserMachine::MACHINE_IDENTIFICATION, serial: 1 };
        let mut hw = MachineHardware{ 
            hw: vec![], 
            identification:  ident,
            ethercat_interface: None,
        };
        hw.hw.push(Hardware::Modbus(  id_modbus ));
        self.modbus_mgrs.push(mgr.clone());
        self.hardware.insert(ident, hw);
    }

      pub fn generate_machine_hardware_from_ethercat(
        &mut self,
        device_infos: &Vec<MachineDeviceInfo>,
        devices_by_address: &mut HashMap<u16, (MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)>,
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
                devices_by_address.remove(&address).map(|hw| {
                    (info.clone(), hw.0, hw.1) // This is a 3-tuple
                })
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