use super::setup::setup;
use crate::app_state::AppState;
use std::{sync::Arc, time::Duration};
use tokio::time::MissedTickBehavior;

pub async fn hotplugging_task(app_state: Arc<AppState>) {
    let mut tokio_interval = tokio::time::interval(Duration::from_millis(100));
    tokio_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        // drop the guard before the await
        {
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

            // get current group
            let group_guard = app_state.ethercat_group.read().await;
            let group_size = match group_guard.as_ref() {
                Some(group) => group.len(),
                None => 0,
            };
            let group_size = group_size as u32;

            if online_devices_count as i32 != group_size as i32 {
                drop(maindevice_guard);
                drop(group_guard);
                let _ = setup(app_state.clone()).await.or_else(|e| {
                    log::error!("Error configuring devices: {:?}", e);
                    Err(e)
                });
            }
        }

        tokio_interval.tick().await;
    }
}
