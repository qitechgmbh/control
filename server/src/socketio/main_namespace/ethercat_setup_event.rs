use crate::{app_state::APP_STATE, ethercat::config::PDI_LEN};
use control_core::{
    identification::{MachineDeviceIdentification, MachineIdentificationUnique},
    socketio::event::Event,
};
use ethercrab::{SubDevicePdi, SubDeviceRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthercatSetupDone {
    pub devices: Vec<DeviceObj>,
    pub machines: Vec<MachineObj>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineObj {
    pub machine_identification_unique: MachineIdentificationUnique,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceObj {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub machine_device_identification: Option<MachineDeviceIdentification>,
    pub subdevice_index: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EthercatSetupEvent {
    Initializing(bool),
    Done(EthercatSetupDone),
    Error(String),
}

impl DeviceObj {
    fn from_subdevice(
        subdevice: &SubDeviceRef<'_, SubDevicePdi<'_, PDI_LEN>>,
        index: usize,
    ) -> Self {
        DeviceObj {
            name: subdevice.name().to_string(),
            configured_address: subdevice.configured_address(),
            product_id: subdevice.identity().product_id,
            revision: subdevice.identity().revision,
            vendor_id: subdevice.identity().vendor_id,
            machine_device_identification: None,
            subdevice_index: index,
        }
    }
}

pub struct EthercatSetupEventBuilder();

impl EthercatSetupEventBuilder {
    const NAME: &'static str = "EthercatSetupEvent";

    pub fn build(&self) -> Event<EthercatSetupEvent> {
        let ethercat_setup_guard = smol::block_on(async {
            return APP_STATE.as_ref().ethercat_setup.read().await;
        });

        let ethercat_setup = match ethercat_setup_guard.as_ref() {
            Some(device) => device,
            None => {
                return Event::new(
                    Self::NAME,
                    EthercatSetupEvent::Error("Ethercat setup not found".to_string()),
                );
            }
        };

        let mut device_objs: Vec<_> = vec![];
        let mut machine_objs: Vec<_> = vec![];

        // add identified devices
        for identified_device_group in ethercat_setup.identified_device_groups.iter() {
            for (j, device) in identified_device_group.iter().enumerate() {
                let subdevice = match ethercat_setup
                    .group
                    .subdevice(&ethercat_setup.maindevice, device.subdevice_index)
                {
                    Ok(subdevice) => subdevice,
                    Err(_) => {
                        return Event::new(
                            Self::NAME,
                            EthercatSetupEvent::Error("Subdevice not found".to_string()),
                        );
                    }
                };
                let mut device_obj = DeviceObj::from_subdevice(&subdevice, device.subdevice_index);
                device_obj.machine_device_identification = match identified_device_group.get(j) {
                    Some(identification) => Some(identification.clone()),
                    None => {
                        return Event::new(
                            Self::NAME,
                            EthercatSetupEvent::Error(format!("Device {} not found", j)),
                        );
                    }
                };
                device_objs.push(device_obj);
            }
        }

        // add machines
        for machine in ethercat_setup.machines.iter() {
            machine_objs.push(MachineObj {
                machine_identification_unique: machine.0.clone(),
                error: match machine.1 {
                    Ok(_) => None,
                    Err(e) => Some(e.to_string()),
                },
            });
        }

        // add unidentified devices
        for device in ethercat_setup.unidentified_devices.iter() {
            let subdevice = &ethercat_setup
                .group
                .subdevice(&ethercat_setup.maindevice, device.subdevice_index)
                .expect("Subdevice not found");
            let device = DeviceObj::from_subdevice(subdevice, device.subdevice_index);
            device_objs.push(device);
        }

        Event::new(
            Self::NAME,
            EthercatSetupEvent::Done(EthercatSetupDone {
                devices: device_objs,
                machines: machine_objs,
            }),
        )
    }

    pub fn initializing(&self) -> Event<EthercatSetupEvent> {
        Event::new(Self::NAME, EthercatSetupEvent::Initializing(true))
    }
}
