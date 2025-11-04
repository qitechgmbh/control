use crate::app_state::AppState;
use bitvec::prelude::*;
use control_core::machines::connection::MachineConnection;
use control_core::realtime::set_core_affinity;
#[cfg(not(feature = "development-build"))]
use control_core::realtime::set_realtime_priority;
use std::sync::Arc;
use std::time::Instant;
use tracing::{instrument, trace_span};

pub fn start_loop_thread(
    app_state: Arc<AppState>,
) -> Result<std::thread::JoinHandle<()>, std::io::Error> {
    // Start control loop
    let res = std::thread::Builder::new()
        .name("loop".to_owned())
        .spawn(move || {
            let _ = set_core_affinity(2);
            #[cfg(not(feature = "development-build"))]
            if let Err(e) = set_realtime_priority() {
                tracing::error!(
                    "[{}::init_loop] Failed to set thread to real-time priority \n{:?}",
                    module_path!(),
                    e
                );
            } else {
                tracing::info!(
                    "[{}::init_loop] Real-time priority set successfully for current thread",
                    module_path!()
                );
            }

            let rt = smol::LocalExecutor::new();
            // Instead of creating a NEW async task every iteration of our realtime txrx loop, create it ONCE
            smol::block_on(async {
                rt.run(async {
                    loop {
                        if let Err(e) = loop_once(app_state.clone()).await {
                            tracing::error!("Loop failed\n{:?}", e);
                            std::process::exit(1);
                        }
                    }
                })
                .await;
            });

            /*
                // If timeouts keep happenning do a spin_sleep, better then thread_sleep and allows for more deterministic timing
                let elapsed = start.elapsed();
                if elapsed < target_cycle {
                    spin_sleep::sleep(target_cycle - elapsed);
                }
            */

            if let Some(last_loop_start) = app_state
                .performance_metrics
                .read_arc_blocking()
                .last_loop_start
            {
                tracing::info!("Failing Loop Took {:?}", last_loop_start.elapsed());
            }
            // Exit the entire program if the Loop fails
            // gets restarted by systemd if running on NixOS, or different distro wtih the same sysd service
            std::process::exit(1);
        });
    return res;
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
    }

    // execute machines
    {
        let span = trace_span!("loop_once_act");
        let _enter = span.enter();

        let machine_guard = app_state.machines.read().await;
        let now = std::time::Instant::now();

        for machine in machine_guard.iter() {
            let connection = &machine.1.lock_blocking().machine_connection;
            if let MachineConnection::Connected(machine) = connection {
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
