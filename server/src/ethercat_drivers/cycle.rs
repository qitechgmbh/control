use super::{actor::Actor, device::Device};
use crate::ethercat::config::{MAX_SUBDEVICES, PDI_LEN};
use ethercrab::{std::ethercat_now, subdevice_group::Op, MainDevice, SubDeviceGroup};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

pub async fn cycle<'maindevice>(
    group_guard: &SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
    maindevice_guard: &MainDevice<'maindevice>,
    devices_guard: &Vec<Option<Arc<RwLock<dyn Device>>>>,
    actors_guard: &Vec<Arc<RwLock<dyn Actor>>>,
    propagation_delays_guard: &Vec<u32>,
    interval: Duration,
) -> Result<(), anyhow::Error> {
    // TS when the TX/RX cycle starts
    let input_ts = ethercat_now();

    // Send/Receive
    group_guard.tx_rx(&maindevice_guard).await?;

    // Prediction when the next TX/RX cycle starts
    let output_ts = input_ts + interval.as_nanos() as u64;

    // Debug timestamp
    let calc_start_ts = ethercat_now();

    // copy inputs to devices
    for (i, subdevice) in group_guard.iter(&maindevice_guard).enumerate() {
        let mut device = match devices_guard[i].as_ref() {
            Some(device) => device.write().await,
            None => continue,
        };
        let input = subdevice.inputs_raw();
        let input_ts = input_ts + propagation_delays_guard[i] as u64;
        device.input_checked(input_ts, input.as_ref())?;
    }

    // execute actors
    let now_ts = ethercat_now();
    for actor in actors_guard.iter() {
        let mut actor = actor.write().await;
        Box::pin(actor.act(now_ts)).await;
    }
    drop(actors_guard);

    // copy outputs from devices
    for (i, subdevice) in group_guard.iter(&maindevice_guard).enumerate() {
        let device = match devices_guard[i].as_ref() {
            Some(device) => device.read().await,
            None => continue,
        };
        let mut output = subdevice.outputs_raw_mut();
        let output_ts = output_ts + propagation_delays_guard[i] as u64;
        device.output_checked(output_ts, output.as_mut())?;
    }

    // calculate the time it took to execute the tick processors
    let calc_end_ts = ethercat_now();
    log::debug!(
        "Calculation took {} us",
        calc_end_ts.saturating_sub(calc_start_ts) / 1000
    );

    // tokio_interval.tick().await;
    Ok(())
}
