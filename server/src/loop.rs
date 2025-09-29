use crate::app_state::AppState;
use crate::panic::{PanicDetails, send_panic};
use bitvec::prelude::*;
use control_core::realtime::{set_core_affinity, set_realtime_priority};
use smol::channel::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{instrument, trace_span};

pub fn init_loop(
    thread_panic_tx: Sender<PanicDetails>,
    app_state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
    // Start control loop
    std::thread::Builder::new()
        .name("loop".to_owned())
        .spawn(move || {
            send_panic(thread_panic_tx.clone());
            let rt = smol::LocalExecutor::new();

            // Set core affinity to third core
            let _ = set_core_affinity(2);

            // Set the thread to real-time priority
            if let Err(e) = set_realtime_priority() {
                tracing::error!(
                    "[{}::init_loop] Failed to set real-time priority \n{:?}",
                    module_path!(),
                    e
                );
            } else {
                tracing::info!(
                    "[{}::init_loop] Real-time priority set successfully",
                    module_path!()
                );
            }

            loop {
                let res = smol::block_on(rt.run(async { loop_once(app_state.clone()).await }));

                if let Err(err) = res {
                    tracing::error!("Loop failed\n{:?}", err);
                    break;
                }
            }
            // Exit the entire program if the Loop fails (gets restarted by systemd if running on NixOS)
            std::process::exit(1);
        })
        .map_err(|e| {
            anyhow::anyhow!(
                "[{}::init_loop] Failed to spawn loop thread\n{:?}",
                module_path!(),
                e
            )
        })?;

    Ok(())
}

#[instrument(skip(app_state))]
pub async fn loop_once<'maindevice>(app_state: Arc<AppState>) -> Result<(), anyhow::Error> {
    // Record cycle start for performance metrics
    {
        let mut metrics = app_state.performance_metrics.write().await;
        metrics.cycle_start();
    }

    let ethercat_setup_guard = app_state.ethercat_setup.read().await;

    // only if we have an ethercat setup
    // - tx/rx cycle
    // - copy inputs to devices
    if let Some(ethercat_setup) = ethercat_setup_guard.as_ref() {
        let span = trace_span!("loop_once_inputs");
        let _enter = span.enter();

        // Measure tx_rx performance
        let txrx_start = Instant::now();
        ethercat_setup
            .group
            .tx_rx(&ethercat_setup.maindevice)
            .await?;
        let txrx_duration = txrx_start.elapsed();

        // Record tx_rx performance metrics
        {
            let mut metrics = app_state.performance_metrics.write().await;
            metrics.record_txrx_time(txrx_duration);
        }

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
            device.input_checked(input_bits).map_err(|e| {
                anyhow::anyhow!(
                    "[{}::loop_once] SubDevice with index {} failed to copy inputs\n{:?}",
                    module_path!(),
                    i,
                    e
                )
            })?;

            // post process inputs
            device.input_post_process().map_err(|e| {
                anyhow::anyhow!(
                    "[{}::loop_once] SubDevice with index {} failed to copy post_process\n{:?}",
                    module_path!(),
                    i,
                    e
                )
            })?;
        }

        #[cfg(feature = "development-build")]
        smol::Timer::after(Duration::from_micros(2000)).await;

        #[cfg(not(feature = "development-build"))]
        smol::Timer::after(Duration::from_micros(500)).await;
    }

    // execute machines
    {
        let span = trace_span!("loop_once_act");
        let _enter = span.enter();

        let machine_guard = app_state.machines.read().await;
        let now = std::time::Instant::now();

        for machine in machine_guard.iter() {
            if let Ok(machine) = machine.1 {
                // if the machine is currenlty locked (likely processing API call)
                // we skip the machine
                if let Some(mut machine_guard) = machine.try_lock() {
                    let span = trace_span!("loop_once_act_machine",);
                    let _enter = span.enter();
                    // execute machine
                    machine_guard.act(now);
                }
            }
        }
    }

    // only if we have an ethercat setup
    // - copy outputs from devices
    if let Some(ethercat_setup) = ethercat_setup_guard.as_ref() {
        let span = trace_span!("loop_once_outputs");
        let _enter = span.enter();

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
            device.output_pre_process().map_err(|e| {
                anyhow::anyhow!(
                    "[{}::loop_once] SubDevice with index {} failed to pre process outputs \n{:?}",
                    module_path!(),
                    i,
                    e
                )
            })?;

            // put outputs into device
            device.output_checked(output_bits).map_err(|e| {
                anyhow::anyhow!(
                    "[{}::loop_once] SubDevice with index {} failed to copy outputs\n{:?}",
                    module_path!(),
                    i,
                    e
                )
            })?;
        }
    }

    Ok(())
}
