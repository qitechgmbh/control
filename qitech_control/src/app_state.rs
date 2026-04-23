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
    MachineMessage,
    machine_identification::{
        DeviceHardwareIdentificationEthercat, DeviceIdentification, DeviceMachineIdentification,
        QiTechMachineIdentificationUnique,
    },
};
use qitech_lib::ethercat_hal::{
    StandardEtherCATController, machine_ident_read::MachineDeviceInfo
};
use socketioxide::{SocketIo, extract::SocketRef};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    RwLock,
    mpsc::{Receiver, Sender},
};

pub struct SocketioSetup {
    pub socketio: RwLock<Option<SocketIo>>,
    pub namespaces: RwLock<Namespaces>,
    pub socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
    // Can/Should be an Arc<RefCell probably
    pub socket_queue_rx: RwLock<Receiver<(SocketRef, Arc<GenericEvent>)>>,
}

/*
    This struct is only written in the main machine loop or during initialization,
    Otherwise it is simply read.
    TODO: then why is it locked and not just read-only?
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
            let mut device_machine_identification = infos.iter()
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

    pub async fn send_machines_event(&self) {
        let event = MachinesEventBuilder().build(self.get_machines_meta().await);
        let mut guard = self.socketio_setup.namespaces.write().await;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
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
        }
    }
}
