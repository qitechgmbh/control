use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use crate::app_state::AppState;
use super::setup::setup;

pub async fn hotplugging_task(app_state: Arc<AppState>) {
    loop {
        // get main device
        let maindevice_guard = app_state.ethercat_master.read().await;
        let maindevice = match maindevice_guard.as_ref() {
            Some(device) => device,
            None => {
                log::error!("MainDevice not initialized");
                continue;
            }
        };

        // check how many devices are online
        let online_devices_count = maindevice
            .count_subdevices()
            .await
            .expect("count subdevices");
        drop(maindevice_guard);

        // get current group
        let group_guard = app_state.ethercat_devices.read().await;
        let group_size = match group_guard.as_ref() {
            Some(group) => group.len(),
            None => 0,
        };
        let group_size = group_size as u32;
        drop(group_guard);

        if online_devices_count as i32 != group_size as i32 {
            let _ = setup(app_state.clone()).await.or_else(|e| {
                log::error!("Error configuring devices: {:?}", e);
                Err(e)
            });
        }

        sleep(Duration::from_millis(100)).await;
    }
}
