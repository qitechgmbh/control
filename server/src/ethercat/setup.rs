use crate::{
    app_state::AppState,
    ethercat::{
        config::{MAX_SUBDEVICES, PDI_LEN},
        mainloop::cycle_task,
    },
    ethercat_drivers::{
        actor::Actor,
        device::{devices_from_subdevice_group, get_device, Device},
        devices::el2008::{EL2008Port, EL2008},
        drivers::digital_output_blinker::DigitalOutputBlinker,
        io::digital_output::DigitalOutputDevice,
    },
    socketio::{event::EventData, messages::ethercat_devices_event::EthercatDevicesEvent},
};
use anyhow::{anyhow, Error, Ok};
use ethercrab::std::ethercat_now;
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

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

    // erase all all setup data
    app_state.ethercat_group.write().await.take();
    app_state.ethercat_devices.write().await.take();
    app_state.ethercat_actors.write().await.take();
    app_state.ethercat_propagation_delays.write().await.take();

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

    let actors: Vec<Arc<RwLock<dyn Actor>>> =
        vec![Arc::new(RwLock::new(DigitalOutputBlinker::new(
            EL2008::digital_output(get_device::<EL2008>(&devices, 2).await?, EL2008Port::Pin1),
            Duration::from_millis(500),
        )))];

    // set all setup data
    let mut ethercat_group_guard = app_state.ethercat_group.write().await;
    *ethercat_group_guard = Some(group_op);
    let mut ethercat_devices_guard = app_state.ethercat_devices.write().await;
    *ethercat_devices_guard = Some(devices);
    let mut ethercat_actors_guard = app_state.ethercat_actors.write().await;
    *ethercat_actors_guard = Some(actors);
    let mut ethercat_propagation_delays_guard = app_state.ethercat_propagation_delays.write().await;
    *ethercat_propagation_delays_guard = Some(propagation_delays);

    // spawn
    tokio::spawn(cycle_task(app_state.clone()));

    // notify client via socketio
    tokio::spawn(async { EthercatDevicesEvent::build().await.emit("main").await });

    Ok(())
}
