use crate::app_state::{EtherCatDeviceMetaData, SharedState};
use control_core::socketio::event::Event;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

    pub async fn build(&self, app_state: Arc<SharedState>) -> Event<EthercatDevicesEvent> {
        Event::new(
            Self::NAME,
            EthercatDevicesEvent::Done(EthercatSetupDone {
                devices: app_state.ethercat_meta_data.read().await.to_vec(),
            }),
        )
    }

    pub fn initializing(&self) -> Event<EthercatDevicesEvent> {
        Event::new(Self::NAME, EthercatDevicesEvent::Initializing(true))
    }
}
