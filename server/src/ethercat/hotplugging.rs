use std::{sync::Arc, time::Duration};

use ethercrab::std::ethercat_now;
use tokio::time::sleep;

use crate::{
    app_state::AppState,
    ethercat::config::{MAX_SUBDEVICES, PDI_LEN},
    socketio::{event::EventData, messages::ethercat_devices_event::EthercatDevicesEvent},
};

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

        // get current group
        let group_guard = app_state.ethercat_devices.read().await;
        let group_size = match group_guard.as_ref() {
            Some(group) => group.len(),
            None => 0,
        };
        drop(group_guard);

        // check how many devices are online
        let online_devices_count = maindevice
            .count_subdevices()
            .await
            .expect("count subdevices");

        if online_devices_count as i32 != group_size as i32 {
            // set group none
            let mut group_guard = app_state.ethercat_devices.write().await;
            group_guard.take();
            drop(group_guard);

            // log difference
            let diff = online_devices_count as i32 - group_size as i32;
            match diff {
                x if x > 0 => log::info!("Detected added Device"),
                x if x < 0 => log::info!("Detected removed Device"),
                _ => log::debug!("Detected nothing!"),
            };

            // initialize all subdevices
            // Fails if DC setup detects a mispatching working copunter, then just try again in loop
            let mut group = None;
            while group.is_none() {
                match maindevice
                    .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
                    .await
                {
                    Ok(g) => group = Some(g),
                    _ => {}
                }
            }
            let group = group.unwrap();
            log::info!("Initialized {} subdevices", group.len());

            // replace the group in the app state
            let mut group_guard = app_state.ethercat_devices.write().await;
            group_guard.replace(group);
            drop(group_guard);

            // notify client via socketio
            EthercatDevicesEvent::emit("main")
        }

        sleep(Duration::from_millis(100)).await;
    }
}
