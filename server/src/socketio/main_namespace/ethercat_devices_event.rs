use std::sync::Arc;

use crate::{app_state::AppState, ethercat::config::PDI_LEN};
use control_core::{
    machines::identification::{DeviceHardwareIdentification, DeviceIdentification},
    socketio::event::Event,
};
use ethercrab::{SubDevicePdi, SubDeviceRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthercatSetupDone {
    pub devices: Vec<DeviceObj>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceObj {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub device_identification: DeviceIdentification,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EthercatDevicesEvent {
    Initializing(bool),
    Done(EthercatSetupDone),
    Error(String),
}

impl DeviceObj {
    fn from_subdevice(
        subdevice: &SubDeviceRef<'_, SubDevicePdi<'_, PDI_LEN>>,
        device_identification: DeviceIdentification,
    ) -> Self {
        DeviceObj {
            name: subdevice.name().to_string(),
            configured_address: subdevice.configured_address(),
            product_id: subdevice.identity().product_id,
            revision: subdevice.identity().revision,
            vendor_id: subdevice.identity().vendor_id,
            device_identification,
        }
    }
}

pub struct EthercatDevicesEventBuilder();

impl EthercatDevicesEventBuilder {
    const NAME: &'static str = "EthercatDevicesEvent";

    pub async fn build(&self, app_state: Arc<AppState>) -> Event<EthercatDevicesEvent> {
        let ethercat_setup_guard = app_state.as_ref().ethercat_setup.read().await;

        let ethercat_setup = match ethercat_setup_guard.as_ref() {
            Some(device) => device,
            None => {
                return Event::new(
                    Self::NAME,
                    EthercatDevicesEvent::Error("Ethercat setup not found".to_string()),
                );
            }
        };

        let mut device_objs: Vec<_> = vec![];

        // add identified devices
        for device in ethercat_setup.devices.iter() {
            // check if its an ethercat device
            let device_hardware_identification_ethercat =
                match device.0.device_hardware_identification {
                    DeviceHardwareIdentification::Ethercat(
                        ref device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    // _ => continue,
                };

            let subdevice = match ethercat_setup.group.subdevice(
                &ethercat_setup.maindevice,
                device_hardware_identification_ethercat.subdevice_index,
            ) {
                Ok(subdevice) => subdevice,
                Err(_) => {
                    return Event::new(
                        Self::NAME,
                        EthercatDevicesEvent::Error("Subdevice not found".to_string()),
                    );
                }
            };

            let device_obj = DeviceObj::from_subdevice(&subdevice, device.0.clone());

            device_objs.push(device_obj);
        }

        Event::new(
            Self::NAME,
            EthercatDevicesEvent::Done(EthercatSetupDone {
                devices: device_objs,
            }),
        )
    }

    pub fn initializing(&self) -> Event<EthercatDevicesEvent> {
        Event::new(Self::NAME, EthercatDevicesEvent::Initializing(true))
    }
}
