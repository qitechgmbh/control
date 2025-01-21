use std::{sync::Arc, time::Duration};

use ethercrab::{
    std::{ethercat_now, tx_rx_task},
    MainDevice, MainDeviceConfig, PduStorage, Timeouts,
};
use tokio::time::sleep;

use crate::{
    app_state::AppState,
    ethercat::config::{MAX_SUBDEVICES, PDI_LEN},
};

use super::config::{MAX_FRAMES, MAX_PDU_DATA};

async fn scan_etehrcat_task(app_state: Arc<AppState>) {
    loop {
        // get main device
        let maindevice_guard = app_state.maindevice.read().await;
        let maindevice = match maindevice_guard.as_ref() {
            Some(device) => device,
            None => {
                log::error!("MainDevice not initialized");
                continue;
            }
        };

        // discover subdevices
        let group = maindevice
            .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
            .await
            .expect("Init");

        log::info!("Discovered {} SubDevices", group.len());

        // replace the group in the app state
        let mut group_guard = app_state.group.write().await;
        group_guard.replace(group);

        sleep(Duration::from_secs(1)).await;
    }
}

pub static PDU_STORAGE: PduStorage<MAX_FRAMES, MAX_PDU_DATA> = PduStorage::new();

pub async fn init_ethercat(app_state: Arc<AppState>) {
    // check interface input
    let interface = "en10";

    let (tx, rx, pdu_loop) = PDU_STORAGE.try_split().expect("can only split once");

    tokio::spawn(tx_rx_task(interface, tx, rx).expect("spawn TX/RX task"));

    // sca
    tokio::spawn(scan_etehrcat_task(app_state.clone()));

    let maindevice = MainDevice::new(
        pdu_loop,
        Timeouts {
            wait_loop_delay: Duration::from_millis(2),
            mailbox_response: Duration::from_millis(1000),
            ..Default::default()
        },
        MainDeviceConfig::default(),
    );

    // add main device to app state
    app_state
        .clone()
        .maindevice
        .write()
        .await
        .replace(maindevice);
}
