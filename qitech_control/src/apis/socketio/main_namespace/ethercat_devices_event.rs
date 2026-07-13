use crate::SharedAppState;
use control_core::socketio::event::Event;
use machine_implementations::machine_identification::DeviceIdentification;
use qitech_lib::ethercat_hal::EtherCATState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

pub struct EcatState(EtherCATState);

impl Into<String> for EcatState {
    fn into(self) -> String {
        // Access the inner type via self.0
        match self.0 {
            EtherCATState::NoInterface => String::from("no interface"),
            EtherCATState::Boot => String::from("booting"),
            EtherCATState::Init => String::from("init"),
            EtherCATState::PreOp => String::from("preop"),
            EtherCATState::PreopPdi => String::from("preoppdi"),
            EtherCATState::Op => String::from("op"),
        }
    }
}

impl From<EtherCATState> for EcatState {
    fn from(value: EtherCATState) -> Self {
        Self(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EthercatDevicesEvent {
    Initializing(bool),
    Done(EthercatSetupDone),
    Error(String),
    State(String),
}

pub struct EthercatDevicesEventBuilder();

impl EthercatDevicesEventBuilder {
    const NAME: &'static str = "EthercatDevicesEvent";

    pub async fn build(&self, app_state: Arc<SharedAppState>) -> Event<EthercatDevicesEvent> {
        Event::new(
            Self::NAME,
            EthercatDevicesEvent::Done(EthercatSetupDone {
                devices: app_state.ethercat_meta_datas.read().await.clone(),
            }),
        )
    }

    pub fn initializing(&self) -> Event<EthercatDevicesEvent> {
        Event::new(Self::NAME, EthercatDevicesEvent::Initializing(true))
    }
}
