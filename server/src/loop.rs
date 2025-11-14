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

            // Wrap the whole async loop in a future
            let loop_future = async {
                let mut ethercat: Option<Box<EthercatSetup>> = None;
                let mut machines: Vec<Box<dyn Machine>> = vec![];

                loop {
                    let msg = match rt_receiver.try_recv() {
                        Ok(msg) => msg,
                        Err(_) => HotThreadMessage::NoMsg,
                    };

                    match msg {
                        HotThreadMessage::NoMsg => {}
                        HotThreadMessage::AddEtherCatSetup(ethercat_setup) => {
                            ethercat = Some(Box::new(ethercat_setup));
                        }
                        HotThreadMessage::WriteMachineDeviceInfo(info_request) => {
                            if let Some(ethercat_setup) = &ethercat {
                                if let Ok(subdevice) = ethercat_setup.group.subdevice(
                                    &ethercat_setup.maindevice,
                                    info_request
                                        .hardware_identification_ethercat
                                        .subdevice_index,
                                ) {
                                    let _res = write_machine_device_identification(
                                        &subdevice,
                                        &ethercat_setup.maindevice,
                                        &info_request.device_machine_identification,
                                    )
                                    .await;
                                }
                            }
                        }
                        HotThreadMessage::DeleteMachine(unique_id) => {
                            machines.retain(|m| m.get_machine_identification_unique() != unique_id);
                        }
                        HotThreadMessage::AddMachines(machine_vec) => {
                            for new_machine in machine_vec {
                                let id = new_machine.get_machine_identification_unique();
                                if !machines
                                    .iter()
                                    .any(|m| m.get_machine_identification_unique() == id)
                                {
                                    machines.push(new_machine);
                                }
                            }
                        }
                    }

                    if let Err(e) = smol::block_on(loop_once(
                        shared_state.clone(),
                        &mut machines,
                        &sleeper,
                        cycle_target,
                        ethercat.as_deref(),
                    )) {
                        tracing::error!("Loop failed\n{:?}", e);
                        break;
                    }
                }
            };
            smol::block_on(loop_future);

            // Exit the entire program if the Loop fails
            // gets restarted by systemd if running on NixOS, or different distro wtih the same sysd service
            std::process::exit(1);
        });
    return res;
}

pub async fn copy_ethercat_inputs(
    ethercat_setup: Option<&EthercatSetup>,
) -> Result<(), anyhow::Error> {
    // only if we have an ethercat setup
    // - tx/rx cycle
    // - copy inputs to devices
    if let Some(ethercat_setup) = ethercat_setup {
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

pub fn execute_machines(machines: &mut Vec<Box<dyn Machine>>) {
    let now = Instant::now();
    for machine in machines.iter_mut() {
        machine.act(now);
    }
}

// No more logging in loop_once
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

    smol::block_on(copy_ethercat_inputs(ethercat_setup))?;
    execute_machines(machines);
    smol::block_on(copy_ethercat_outputs(ethercat_setup))?;

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
