use crate::app_state::{EthercatSetup, HotThreadMessage, SharedState};
use bitvec::prelude::*;
use control_core::realtime::set_core_affinity;
#[cfg(not(feature = "development-build"))]
use control_core::realtime::set_realtime_priority;
use machines::Machine;
use machines::machine_identification::write_machine_device_identification;
use smol::channel::Receiver;
use spin_sleep::SpinSleeper;
use std::time::Instant;
use std::{sync::Arc, time::Duration};
use tracing::{instrument, trace_span};

// 300 us loop cycle target
// SharedState is mostly read from and rarely locked, but does not contain any machine,ethercat devices etc
pub fn start_loop_thread(
    shared_state: Arc<SharedState>,
    rt_receiver: Receiver<HotThreadMessage>,
    cycle_target: Duration,
) -> Result<std::thread::JoinHandle<()>, std::io::Error> {
    // Start control loop
    let res = std::thread::Builder::new()
        .name("loop".to_owned())
        .spawn(move || {
            let rt_receiver = rt_receiver.to_owned();
            let sleeper =
                SpinSleeper::new(3_333_333) // frequency in Hz ~ 1 / 300µs, Basically specifies the accuracy of our sleep
                    .with_spin_strategy(spin_sleep::SpinStrategy::YieldThread);

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
            let mut ethercat: Option<Box<EthercatSetup>> = None;
            // We need the vec in a box due to Mutably borrowing
            let mut machines: Vec<Box<dyn Machine>> = vec![];

            smol::block_on(async {
                rt.run(async {
                    loop {
                        let res = rt_receiver.try_recv();
                        let msg = match res {
                            Ok(msg) => msg,
                            Err(_) => HotThreadMessage::NoMsg,
                        };

                        match msg {
                            HotThreadMessage::NoMsg => (),
                            HotThreadMessage::AddEtherCatSetup(ethercat_setup) => {
                                ethercat = Some(Box::new(ethercat_setup));
                            }
                            HotThreadMessage::WriteMachineDeviceInfo(
                                machine_device_info_request,
                            ) => {
                                match &ethercat {
                                    Some(ethercat_setup) => {
                                        match ethercat_setup.group.subdevice(
                                                    &ethercat_setup.maindevice,
                                                    machine_device_info_request.hardware_identification_ethercat.subdevice_index,
                                            ) {
                                                Ok(subdevice) => {
                                                    let _ = write_machine_device_identification(
                                                        &subdevice,
                                                        &ethercat_setup.maindevice,
                                                        &machine_device_info_request.device_machine_identification,
                                                    ).await;                                                    
                                                },
                                                Err(_) => (),
                                            };
                                    },
                                    None => (),
                                };
                            },                                                                                        
                            HotThreadMessage::DeleteMachine(machine_identification_unique) => {
                                machines.retain(|m| {
                                    m.get_machine_identification_unique()
                                        != machine_identification_unique
                                });
                            }
                            HotThreadMessage::AddMachines(machine_vec) => {
                                for new_machine in machine_vec {
                                    let id = new_machine.get_machine_identification_unique();
                                    // Only add if a machine with the same ID doesn't exist
                                    if !machines
                                        .iter()
                                        .any(|m| m.get_machine_identification_unique() == id)
                                    {
                                        machines.push(new_machine);
                                    }
                                }
                            }
                        };

                        if let Err(e) = loop_once(
                            shared_state.clone(),
                            &mut machines,
                            &sleeper,
                            cycle_target,
                            ethercat.as_deref(),
                        )
                        .await
                        {
                            tracing::error!("Loop failed\n{:?}", e);
                            break;
                        }
                    }
                })
                .await;
            });

            if let Some(last_loop_start) = shared_state
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

pub async fn copy_ethercat_inputs(
    ethercat_setup: Option<&EthercatSetup>,
) -> Result<(), anyhow::Error> {
    //println!("copy_ethercat_intputs");
    // only if we have an ethercat setup
    // - tx/rx cycle
    // - copy inputs to devices

    if let Some(ethercat_setup) = ethercat_setup {
        let span = trace_span!("loop_once_inputs");
        let _enter = span.enter();
        // Measure tx_rx performance
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
    Ok(())
}

pub async fn copy_ethercat_outputs(
    ethercat_setup: Option<&EthercatSetup>,
) -> Result<(), anyhow::Error> {
    if let Some(ethercat_setup) = ethercat_setup {
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

pub async fn execute_machines(machines: &mut Vec<Box<dyn Machine>>) {
    let now = Instant::now();
    for machine in machines.iter_mut() {
        let span = trace_span!("loop_once_act_machine",);
        let _enter = span.enter();
        machine.act(now);
    }
}

#[instrument(skip(app_state, machines))]
pub async fn loop_once<'maindevice>(
    app_state: Arc<SharedState>,
    machines: &mut Vec<Box<dyn Machine>>,
    sleeper: &SpinSleeper,
    cycle_target: Duration,
    ethercat_setup: Option<&EthercatSetup>,
) -> Result<(), anyhow::Error> {
    let loop_once_start = std::time::Instant::now();
    // Record cycle start for performance metrics
    {
        let mut metrics = app_state.performance_metrics.write().await;
        metrics.cycle_start();
    }
    {
        let txrx_start = Instant::now();
        copy_ethercat_inputs(ethercat_setup).await?;
        let txrx_duration = txrx_start.elapsed();
        let mut metrics = app_state.performance_metrics.write().await;
        metrics.record_txrx_time(txrx_duration);
    }

    // execute machines
    {
        let span = trace_span!("loop_once_act");
        let _enter = span.enter();
        execute_machines(machines).await;
    }

    {
        copy_ethercat_outputs(ethercat_setup).await?;
    }

    if ethercat_setup.is_some() {
        // spin_sleep so we have a cycle time of ~300us
        // This does push usage to 100% if completely busy, but provides much better accuracy then thread sleep or async sleep
        sleeper.sleep_until(loop_once_start + cycle_target);
    } else {
        // if we dont have an ethercat setup or other rt relevant stuff do the "worse" async sleep or later if we get rid of async thread::sleep or yielding
        // We do this, so that when no rt relevant code runs the cpu doesnt spin at 100% for no reason
        let loop_duration = loop_once_start.elapsed();
        if cycle_target > loop_once_start.elapsed() {
            smol::Timer::after(cycle_target - loop_duration).await;
        }
    }
    Ok(())
}
