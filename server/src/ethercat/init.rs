use std::{sync::Arc, time::Duration};

use ethercrab::{std::tx_rx_task, MainDevice, MainDeviceConfig, PduStorage, Timeouts};

use crate::app_state::AppState;

use super::{
    config::{MAX_FRAMES, MAX_PDU_DATA},
    hotplugging::hotplugging_task,
};

pub async fn init_ethercat(app_state: Arc<AppState>) {
    let interface = "en10";
    let pdu_storage = Box::new(PduStorage::<MAX_FRAMES, MAX_PDU_DATA>::new());
    let pdu_storage = Box::leak(pdu_storage);
    let (tx, rx, pdu_loop) = pdu_storage.try_split().expect("can only split once");

    // build main device
    let maindevice = MainDevice::new(
        pdu_loop,
        Timeouts {
            wait_loop_delay: Duration::from_millis(2),
            mailbox_response: Duration::from_millis(1000),
            ..Default::default()
        },
        MainDeviceConfig::default(),
    );

    // replace maindevice in app state
    let mut maindevice_guard = app_state.ethercat_master.write().await;
    maindevice_guard.replace(maindevice);
    drop(maindevice_guard);

    // check interface input
    tokio::spawn(tx_rx_task(&interface, tx, rx).expect("spawn TX/RX task"));

    // scan ethercat task
    tokio::spawn(hotplugging_task(app_state.clone()));
}
