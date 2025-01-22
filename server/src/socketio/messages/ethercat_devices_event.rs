use serde::{Deserialize, Serialize};

use crate::{
    app_state::APP_STATE,
    socketio::event::{Event, EventData, EventType},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthercatDevicesEvent {
    pub devices: Vec<Device>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub address: u16,
    pub name: String,
}
const EVENT: &str = "EthercatDevicesEvent";

impl EventData for EthercatDevicesEvent {
    async fn build() -> Event<Self> {
        let maindevice_guard = APP_STATE.as_ref().ethercat_master.read().await;
        let maindevice = match maindevice_guard.as_ref() {
            Some(device) => device,
            None => {
                return Event::error(EVENT.to_string(), "MainDevice not initialized".to_string());
            }
        };

        let mut group_guard = APP_STATE.ethercat_devices.write().await;
        let group = match group_guard.as_mut() {
            Some(group) => group,
            None => {
                return Event::error(
                    EVENT.to_string(),
                    "SubDeviceGroup not initialized".to_string(),
                );
            }
        };

        Event::data(
            EVENT.to_string(),
            EthercatDevicesEvent {
                devices: (*group)
                    .iter(&maindevice)
                    .map(|subdevice| Device {
                        address: subdevice.configured_address(),
                        name: subdevice.name().to_string(),
                    })
                    .collect::<Vec<_>>(),
            },
        )
    }

    fn to_message_type(message: Event<Self>) -> EventType {
        EventType::EthercatDevicesEvent(message)
    }
}
