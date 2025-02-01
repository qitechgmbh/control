use super::{actor::Actor, device::Device};
use crate::ethercat::config::{MAX_SUBDEVICES, PDI_LEN};
use ethercrab::{std::ethercat_now, subdevice_group::Op, MainDevice, SubDeviceGroup};
use std::{sync::Arc, time::Duration};

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

    // copy inputs to devices
    let ts_1 = ethercat_now();

    for (i, subdevice) in group_guard.iter(&maindevice_guard).enumerate() {
        let mut device = match devices_guard[i].as_ref() {
            Some(device) => device.write().await,
            None => continue,
        };
        let input_ts = input_ts + propagation_delays_guard[i] as u64;
        let output_ts = output_ts + propagation_delays_guard[i] as u64;
        device.ts(input_ts, output_ts);
        let input = subdevice.inputs_raw();
        device.input_checked(input.as_ref())?;
    }
    // let ts_2 = ethercat_now();
    // log::info!("-> Copy inputs took {} ns", ts_2 - ts_1);

    // execute actors
    let now_ts = ethercat_now();
    for actor in actors_guard.iter() {
        let mut actor = actor.write().await;
        Box::pin(actor.act(now_ts)).await;
    }
    // let ts_3 = ethercat_now();
    // log::info!("-> Actors took {} ns", ts_3 - ts_2);

    // copy outputs from devices
    for (i, subdevice) in group_guard.iter(&maindevice_guard).enumerate() {
        let device = match devices_guard[i].as_ref() {
            Some(device) => device.read().await,
            None => continue,
        };
        let mut output = subdevice.outputs_raw_mut();
        device.output_checked(output.as_mut())?;
    }

    // calculate the time it took to execute the tick processors
    let ts_4 = ethercat_now();
    log::info!("-> Cycle function took {} ns", ts_4 - ts_1);
    // tokio_interval.tick().await;
    Ok(())
}
