use crate::{
    app_state::APP_STATE,
    socketio::event::{Event, EventData, EventType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthercatDevicesEvent {
    pub devices: Vec<Device>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub address: u16,
    pub name: String,
    pub alias_address: u16,
    pub configured_address: u16,
    pub dc_support: bool,
    pub propagation_delay: u32,
    pub product_id: u32,
    pub revision: u32,
    pub serial: u32,
    pub vendor_id: u32,
}
const EVENT: &str = "EthercatDevicesEvent";

impl EventData for EthercatDevicesEvent {
    async fn build() -> Event<Self> {
        let ethercat_setup_guard = APP_STATE.as_ref().ethercat_setup.read();
        let ethercat_setup = match ethercat_setup_guard.as_ref() {
            Some(device) => device,
            None => {
                return Event::error(
                    EVENT.to_string(),
                    "EthercatSetup not initialized".to_string(),
                );
            }
        };

        Event::data(
            EVENT.to_string(),
            EthercatDevicesEvent {
                devices: (ethercat_setup.group)
                    .iter(&ethercat_setup.maindevice)
                    .map(|subdevice| {
                        log::info!("Subdevice: {:?}", subdevice.identity());
                        return Device {
                            address: subdevice.configured_address(),
                            name: subdevice.name().to_string(),
                            alias_address: subdevice.alias_address(),
                            configured_address: subdevice.configured_address(),
                            dc_support: subdevice.dc_support().any(),
                            propagation_delay: subdevice.propagation_delay(),
                            product_id: subdevice.identity().product_id,
                            revision: subdevice.identity().revision,
                            serial: subdevice.identity().serial,
                            vendor_id: subdevice.identity().vendor_id,
                        };
                    })
                    .collect::<Vec<_>>(),
            },
        )
    }

    fn build_warning(warning: String) -> Event<Self> {
        Event::warning(EVENT.to_string(), warning)
    }

    fn to_event_type(message: Event<Self>) -> EventType {
        EventType::EthercatDevicesEvent(message)
    }
}
