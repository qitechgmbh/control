use crate::ethercat_drivers::cycle::cycle;
use anyhow::anyhow;
use ethercrab::std::ethercat_now;
use std::sync::Arc;
use std::time::Duration;

use crate::app_state::AppState;

async fn cycle_task_failing(app_state: Arc<AppState>) -> Result<(), anyhow::Error> {
    let interval = Duration::from_nanos(1);

    // let mut tokio_interval = tokio::time::interval(interval);
    // tokio_interval.set_missed_tick_behavior(MissedTickBehavior::Burst);

    loop {
        // drop the guard before the await
        {
            // let ts_1 = ethercat_now();
            let group_guard = app_state.ethercat_group.read().await;
            let group_guard = group_guard
                .as_ref()
                .ok_or_else(|| anyhow!("Maindevice not initialized"))?;
            let maindevice_guard = app_state.ethercat_master.read().await;
            let maindevice_guard = maindevice_guard
                .as_ref()
                .ok_or_else(|| anyhow!("Maindevice not initialized"))?;
            let devices_guard = app_state.ethercat_devices.read().await;
            let devices_guard = devices_guard
                .as_ref()
                .ok_or_else(|| anyhow!("Devices not initialized"))?;
            let propagation_delays_guard = app_state.ethercat_propagation_delays.read().await;
            let propagation_delays_guard = propagation_delays_guard
                .as_ref()
                .ok_or_else(|| anyhow!("Propagation delays not initialized"))?;
            let actors_guard = app_state.ethercat_actors.read().await;
            let actors_guard = actors_guard
                .as_ref()
                .ok_or_else(|| anyhow!("Actors not initialized"))?;
            let ts_2 = ethercat_now();
            // log::info!("Read guards took {} ns", ts_2 - ts_1);

            let ts_1 = ethercat_now();
            cycle(
                group_guard,
                maindevice_guard,
                devices_guard,
                actors_guard,
                propagation_delays_guard,
                interval,
            )
            .await?;
            let ts_2 = ethercat_now();
            log::info!("Cycle await took {} ns", ts_2 - ts_1);
        }

        // tokio_interval.tick().await;
    }
}

pub async fn cycle_task(app_state: Arc<AppState>) {
    loop {
        if let Err(e) = cycle_task_failing(app_state.clone()).await {
            log::error!("Error in cycle task: {:?}", e);
        }
    }
}
