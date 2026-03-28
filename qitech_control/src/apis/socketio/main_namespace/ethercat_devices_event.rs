use control_core::socketio::event::Event;
use machine_implementations::machine_identification::{DeviceIdentification};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{SharedAppState};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EtherCatDeviceMetaData {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub device_identification: DeviceIdentification,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EthercatSetupDone {
    pub devices: Vec<EtherCatDeviceMetaData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EthercatDevicesEvent {
    Initializing(bool),
    Done(EthercatSetupDone),
    Error(String),
}

pub struct EthercatDevicesEventBuilder();

impl EthercatDevicesEventBuilder {
    const NAME: &'static str = "EthercatDevicesEvent";

    pub async fn build(&self, app_state: Arc<SharedAppState>) -> Event<EthercatDevicesEvent> {
        Event::new(
            Self::NAME,
            EthercatDevicesEvent::Done(EthercatSetupDone {
                devices: app_state.ethercat_meta_datas.clone(),
            }),
        )
    }

    pub fn initializing(&self) -> Event<EthercatDevicesEvent> {
        Event::new(Self::NAME, EthercatDevicesEvent::Initializing(true))
    }
}
