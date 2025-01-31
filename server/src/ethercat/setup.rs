use crate::{
    app_state::AppState,
    ethercat::config::{MAX_SUBDEVICES, PDI_LEN},
    ethercat_drivers::{
        devices::{
            devices_from_subdevice_group, downcast_device,
            el2008::{EL2008Port, EL2008},
            Device,
        },
        drivers::digital_output_blinker::DigitalOutputBlinker,
        tick::Tick,
    },
    socketio::{event::EventData, messages::ethercat_devices_event::EthercatDevicesEvent},
};
use anyhow::{anyhow, Error, Ok};
use ethercrab::std::ethercat_now;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{sync::RwLock, time::MissedTickBehavior};

pub async fn setup(app_state: Arc<AppState>) -> Result<(), Error> {
    // notify client via socketio
    tokio::spawn(async {
        EthercatDevicesEvent::build_warning("Configuring Devices...".to_string())
            .emit("main")
            .await
    });

    // get main device
    let maindevice_guard = app_state.ethercat_master.read().await;
    let maindevice = maindevice_guard
        .as_ref()
        .ok_or(anyhow!("MainDevice not initialized"))?;

    // set group none
    app_state.ethercat_devices.write().await.take();

    // initialize all subdevices
    // Fails if DC setup detects a mispatching working copunter, then just try again in loop
    let group = loop {
        let group = maindevice
            .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
            .await;
        // if ok return
        if group.is_ok() {
            break group.unwrap();
        }
    };

    log::info!("Initialized {} subdevices", group.len());

    // put group in op state
    let group_op = group.into_op(&maindevice).await?;

    let propagation_delays: Vec<u32> = group_op
        .iter(maindevice)
        .map(|subdevice| subdevice.propagation_delay())
        .collect();

    // create devices
    let devices: Vec<Option<Arc<RwLock<dyn Device>>>> =
        devices_from_subdevice_group(&group_op, maindevice);

    // create EL2008 device
    let el2008 = downcast_device::<EL2008>(devices[2].as_ref().unwrap().clone()).await?;

    let tick_processors: Vec<Arc<RwLock<dyn Tick>>> =
        vec![Arc::new(RwLock::new(DigitalOutputBlinker::new(
            EL2008::digital_output(el2008.clone(), EL2008Port::Pin1),
            Duration::from_millis(500),
        )))];

    let mut tick_interval = tokio::time::interval(Duration::from_millis(5));
    tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let shutdown = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&shutdown))
        .expect("Register hook");

    loop {
        // Graceful shutdown on Ctrl + C
        if shutdown.load(Ordering::Relaxed) {
            log::info!("Shutting down...");
            break;
        }
        // the ts the loop started and inputs were read
        let input_ts = ethercat_now();
        group_op.tx_rx(&maindevice).await.expect("TX/RX");
        let output_ts = input_ts + tick_interval.period().as_nanos() as u64;
        let calc_start_ts = ethercat_now();

        // copy inputs to devices
        for (i, subdevice) in group_op.iter(&maindevice).enumerate() {
            let mut device = match devices[i].as_ref() {
                Some(device) => device.write().await,
                None => continue,
            };
            let input = subdevice.inputs_raw();
            let input_ts = input_ts + propagation_delays[i] as u64;
            device.input(input_ts, input.as_ref())?;
        }

        // execute tick processors
        let now_ts = ethercat_now();
        for tick_processor in tick_processors.iter() {
            let mut tick_processor = tick_processor.write().await;
            Box::pin(tick_processor.tick(now_ts)).await;
        }

        // copy outputs from devices
        for (i, subdevice) in group_op.iter(&maindevice).enumerate() {
            let device = match devices[i].as_ref() {
                Some(device) => device.read().await,
                None => continue,
            };
            let mut output = subdevice.outputs_raw_mut();
            let output_ts = output_ts + propagation_delays[i] as u64;
            device.output(output_ts, output.as_mut())?;
        }

        let calc_end_ts = ethercat_now();
        log::debug!(
            "Calculation took {} ns",
            calc_end_ts.saturating_sub(calc_start_ts)
        );

        tick_interval.tick().await;
    }

    // replace the group in the app state
    let mut group_guard = app_state.ethercat_devices.write().await;
    group_guard.replace(group_op);

    // notify client via socketio
    tokio::spawn(async { EthercatDevicesEvent::build().await.emit("main").await });

    Ok(())
}
