use crate::app_state::EthercatSetup;
use ethercrab::std::ethercat_now;
use std::time::Duration;

pub fn pdu_once<'maindevice>(
    setup: &EthercatSetup,
    interval: Duration,
    rt: &tokio::runtime::Runtime,
) -> Result<(), anyhow::Error> {
    // TS when the TX/RX cycle starts
    let input_ts = ethercat_now();

    // Prediction when the next TX/RX cycle starts
    let output_ts = input_ts + interval.as_nanos() as u64;

    let ts1 = ethercat_now();
    rt.block_on(async { setup.group.tx_rx(&setup.maindevice).await })?;
    let ts2 = ethercat_now();
    log::info!("TX/RX took {} ns", ts2 - ts1);

    // copy inputs to devices
    for (i, subdevice) in setup.group.iter(&setup.maindevice).enumerate() {
        let mut device = match setup.devices[i].as_ref() {
            Some(device) => device.write(),
            None => continue,
        };
        let input_ts = input_ts;
        let output_ts = output_ts;
        device.ts(input_ts, output_ts);
        let input = subdevice.inputs_raw();
        device.input_checked(input.as_ref())?;
    }
    let ts3 = ethercat_now();
    log::info!("Input took {} ns", ts3 - ts2);

    // execute actors
    for actor in setup.actors.iter() {
        let mut actor = actor.write();
        actor.act(output_ts);
    }
    let ts4 = ethercat_now();
    log::info!("Actors took {} ns", ts4 - ts3);

    // copy outputs from devices
    for (i, subdevice) in setup.group.iter(&setup.maindevice).enumerate() {
        let device = match setup.devices[i].as_ref() {
            Some(device) => device.read(),
            None => continue,
        };
        let mut output = subdevice.outputs_raw_mut();
        device.output_checked(output.as_mut())?;
    }
    let ts5 = ethercat_now();
    log::info!(
        "Output took {} ns and total PDU cycle took {} ns",
        ts5 - ts4,
        ts5 - ts1
    );

    Ok(())
}
