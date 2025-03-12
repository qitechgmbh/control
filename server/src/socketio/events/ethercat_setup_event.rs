use crate::{
    app_state::{MachineInfo, APP_STATE},
    ethercat::{config::PDI_LEN, device_identification::MachineDeviceIdentification},
    socketio::event::{Event, EventData, EventType},
};
use ethercrab::{SubDevicePdi, SubDeviceRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthercatSetupEvent {
    pub devices: Vec<DeviceObj>,
    pub machine_infos: Vec<MachineInfo>,
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

const EVENT: &str = "EthercatSetupEvent";

impl EventData for EthercatSetupEvent {
    async fn build() -> Event<Self> {
        let ethercat_setup_guard = APP_STATE.as_ref().ethercat_setup.read().await;
        let ethercat_setup = match ethercat_setup_guard.as_ref() {
            Some(device) => device,
            None => {
                return Event::error(
                    EVENT.to_string(),
                    "EthercatSetup not initialized".to_string(),
                );
            }
        };

        let mut device_objs: Vec<_> = vec![];
        let mut machine_infos: Vec<_> = vec![];

        // add identified devices
        for (i, identified_device_group) in
            ethercat_setup.identified_device_groups.iter().enumerate()
        {
            log::info!("Device Group: {:?}", identified_device_group);
            for (j, device) in identified_device_group.iter().enumerate() {
                log::info!("Device: {:?}", device);
                let subdevice = match ethercat_setup
                    .group
                    .subdevice(&ethercat_setup.maindevice, device.subdevice_index)
                {
                    Ok(subdevice) => subdevice,
                    Err(_) => {
                        return Event::error(EVENT.to_string(), "Subdevice not found".to_string());
                    }
                };
                let mut device_obj = DeviceObj::from_subdevice(&subdevice, device.subdevice_index);
                device_obj.machine_device_identification = match identified_device_group.get(j) {
                    Some(identification) => Some(identification.clone()),
                    None => {
                        return Event::error(EVENT.to_string(), format!("Device {} not found", j));
                    }
                };
                log::info!("Device: {:?}", device_obj);
                device_objs.push(device_obj);
            }

            // add machine

            let machine_info = match ethercat_setup.machine_infos.get(i) {
                Some(machine) => machine,
                None => {
                    return Event::error(EVENT.to_string(), "Machine not found".to_string());
                }
            };
            machine_infos.push(machine_info.clone());
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

        Event::data(
            EVENT.to_string(),
            EthercatSetupEvent {
                devices: device_objs,
                machine_infos: machine_infos,
            },
        )
    }

    fn build_warning(warning: String) -> Event<Self> {
        Event::warning(EVENT.to_string(), warning)
    }

    fn to_event_type(message: Event<Self>) -> EventType {
        EventType::EthercatSetupEvent(message)
    }
}
