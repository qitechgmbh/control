use crate::ethercat_drivers::cycle::cycle;
use anyhow::anyhow;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::MissedTickBehavior;

use crate::app_state::AppState;

async fn cycle_task_failing(app_state: Arc<AppState>) -> Result<(), anyhow::Error> {
    let interval = Duration::from_millis(1);

    let mut tokio_interval = tokio::time::interval(Duration::from_millis(5));
    tokio_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        // drop the guard before the await
        {
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

            cycle(
                group_guard,
                maindevice_guard,
                devices_guard,
                actors_guard,
                propagation_delays_guard,
                interval,
            )
            .await?;
        }

        tokio_interval.tick().await;
    }
}

pub async fn cycle_task(app_state: Arc<AppState>) {
    loop {
        if let Err(e) = cycle_task_failing(app_state.clone()).await {
            log::error!("Error in cycle task: {:?}", e);
        }
    }
}
