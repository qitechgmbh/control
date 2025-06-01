use crate::app_state::AppState;
use crate::panic::send_panic;
use bitvec::prelude::*;
use control_core::helpers::loop_trottle::LoopThrottle;
use smol::channel::Sender;
use std::sync::Arc;
use std::time::Duration;

pub fn init_loop(
    thread_panic_tx: Sender<&'static str>,
    app_state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
    // Start control loop
    std::thread::Builder::new()
        .name("LoopThread".to_owned())
        .spawn(move || {
            send_panic("LoopThread", thread_panic_tx.clone());
            let rt = smol::LocalExecutor::new();
            let mut throttle = LoopThrottle::new(Duration::from_millis(1), 1, None);
            loop {
                let res = smol::block_on(rt.run(async {
                    throttle.sleep().await;
                    loop_once(app_state.clone()).await
                }));
                if let Err(err) = res {
                    tracing::error!("Loop failed\n{:?}", err);
                    break;
                }
            }
            // loop should never exit, but if it does, we send a panic message
            // this causes the server to exit (and restarted by systemd if running on NixOS)
            panic!(
                "[{}::init_loop] Loop thread exited unexpectedly",
                module_path!()
            );
        })
        .or_else(|e| {
            Err(anyhow::anyhow!(
                "[{}::init_loop] Failed to spawn loop thread\n{:?}",
                module_path!(),
                e
            ))
        })?;

    Ok(())
}

pub async fn loop_once<'maindevice>(app_state: Arc<AppState>) -> Result<(), anyhow::Error> {
    let ethercat_setup_guard = app_state.ethercat_setup.read().await;

    // only if we have an ethercat setup
    // - tx/rx cycle
    // - copy inputs to devices
    if let Some(ethercat_setup) = ethercat_setup_guard.as_ref() {
        // TX/RX cycle
        ethercat_setup
            .group
            .tx_rx(&ethercat_setup.maindevice)
            .await?;

        // copy inputs to devices
        for (i, subdevice) in ethercat_setup
            .group
            .iter(&ethercat_setup.maindevice)
            .enumerate()
        {
            // retrieve inputs
            let input = subdevice.inputs_raw();
            let input_bits = input.view_bits::<Lsb0>();

            // get device
            let mut device = ethercat_setup.devices[i].1.as_ref().write().await;

            // check if the device is used
            if !device.is_used() {
                // if the device is not used, we skip it
                continue;
            }

            // put inputs into device
            device.input_checked(input_bits).or_else(|e| {
                Err(anyhow::anyhow!(
                    "[{}::loop_once] SubDevice with index {} failed to copy inputs\n{:?}",
                    module_path!(),
                    i,
                    e
                ))
            })?;

            // post process inputs
            device.input_post_process().or_else(|e| {
                Err(anyhow::anyhow!(
                    "[{}::loop_once] SubDevice with index {} failed to copy post_process\n{:?}",
                    module_path!(),
                    i,
                    e
                ))
            })?;
        }
    }

    // execute machines
    let machine_guard = app_state.machines.read().await;
    let now = std::time::Instant::now();
    for machine in machine_guard.iter() {
        if let Ok(machine) = machine.1 {
            // if the machine is currenlty locked (likely processing API call)
            // we skip the machine
            match machine.try_lock() {
                Some(mut machine_guard) => {
                    // execute machine
                    machine_guard.act(now).await;
                }
                None => {}
            }
        }
    }
    drop(machine_guard);

    // only if we have an ethercat setup
    // - copy outputs from devices
    if let Some(ethercat_setup) = ethercat_setup_guard.as_ref() {
        // copy outputs from devices
        for (i, subdevice) in ethercat_setup
            .group
            .iter(&ethercat_setup.maindevice)
            .enumerate()
        {
            // get output buffer for device
            let mut output = subdevice.outputs_raw_mut();
            let output_bits = output.view_bits_mut::<Lsb0>();

            // get device
            let mut device = ethercat_setup.devices[i].1.as_ref().write().await;

            // check if the device is used
            if !device.is_used() {
                // if the device is not used, we skip it
                continue;
            }

            // pre process outputs
            device.output_pre_process().or_else(|e| {
                Err(anyhow::anyhow!(
                    "[{}::loop_once] SubDevice with index {} failed to pre process outputs \n{:?}",
                    module_path!(),
                    i,
                    e
                ))
            })?;

            // put outputs into device
            device.output_checked(output_bits).or_else(|e| {
                Err(anyhow::anyhow!(
                    "[{}::loop_once] SubDevice with index {} failed to copy outputs\n{:?}",
                    module_path!(),
                    i,
                    e
                ))
            })?;
        }
    }

    Ok(())
}
