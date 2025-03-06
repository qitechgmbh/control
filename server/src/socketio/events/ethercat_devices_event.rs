use crate::{
    app_state::APP_STATE,
    ethercat::{config::PDI_LEN, device_identification::MachineDeviceIdentification},
    socketio::event::{Event, EventData, EventType},
};
use ethercrab::{SubDevicePdi, SubDeviceRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthercatDevicesEvent {
    pub devices: Vec<Device>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub machine_device_identification: Option<MachineDeviceIdentification>,
    pub subdevice_index: usize,
}

impl Device {
    fn from_subdevice(
        subdevice: &SubDeviceRef<'_, SubDevicePdi<'_, PDI_LEN>>,
        index: usize,
    ) -> Self {
        Device {
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

const EVENT: &str = "EthercatDevicesEvent";

impl EventData for EthercatDevicesEvent {
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

        let mut devices: Vec<_> = vec![];

        // add identified devices
        for device_group in ethercat_setup.device_groups.iter() {
            log::info!("Device Group: {:?}", device_group);
            for (i, device) in device_group.iter().enumerate() {
                log::info!("Device: {:?}", device);
                let subdevice = &ethercat_setup
                    .group
                    .subdevice(&ethercat_setup.maindevice, device.subdevice_index)
                    .expect("Subdevice not found");
                let mut device = Device::from_subdevice(subdevice, device.subdevice_index);
                device.machine_device_identification = Some(
                    device_group
                        .get(i)
                        .expect(&format!("Device {} not found", i))
                        .clone(),
                );
                log::info!("Device: {:?}", device);
                devices.push(device);
            }
        }

        // add unidentified devices
        for device in ethercat_setup.undetected_devices.iter() {
            let subdevice = &ethercat_setup
                .group
                .subdevice(&ethercat_setup.maindevice, device.subdevice_index)
                .expect("Subdevice not found");
            let device = Device::from_subdevice(subdevice, device.subdevice_index);
            devices.push(device);
        }

        Event::data(EVENT.to_string(), EthercatDevicesEvent { devices: devices })
    }

    fn build_warning(warning: String) -> Event<Self> {
        Event::warning(EVENT.to_string(), warning)
    }

    fn to_event_type(message: Event<Self>) -> EventType {
        EventType::EthercatDevicesEvent(message)
    }
}
