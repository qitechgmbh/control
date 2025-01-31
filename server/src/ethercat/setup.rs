use crate::{
    app_state::AppState,
    ethercat::{
        config::{MAX_SUBDEVICES, PDI_LEN},
        mainloop::cycle_task,
    },
    ethercat_drivers::{
        actor::Actor,
        device::{devices_from_subdevice_group, get_device, Device},
        devices::{
            el2008::{EL2008Port, EL2008},
            el2809::{EL2809Port, EL2809},
        },
        drivers::digital_output_blinkers::DigitalOutputBlinkers,
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

    let actors: Vec<Arc<RwLock<dyn Actor>>> = vec![DigitalOutputBlinkers::new_arc_rwlock(
        vec![
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin1,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin2,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin3,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin4,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin5,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin6,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin7,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin8,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin16,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin15,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin14,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin13,
            )),
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin12,
            )),
            None,
            None,
            Some(EL2008::digital_output(
                get_device::<EL2008>(&devices, 2).await?,
                EL2008Port::Pin7,
            )),
            Some(EL2008::digital_output(
                get_device::<EL2008>(&devices, 2).await?,
                EL2008Port::Pin8,
            )),
            Some(EL2008::digital_output(
                get_device::<EL2008>(&devices, 2).await?,
                EL2008Port::Pin6,
            )),
            Some(EL2008::digital_output(
                get_device::<EL2008>(&devices, 2).await?,
                EL2008Port::Pin4,
            )),
            Some(EL2008::digital_output(
                get_device::<EL2008>(&devices, 2).await?,
                EL2008Port::Pin2,
            )),
            Some(EL2008::digital_output(
                get_device::<EL2008>(&devices, 2).await?,
                EL2008Port::Pin1,
            )),
            None,
            None,
            Some(EL2809::digital_output(
                get_device::<EL2809>(&devices, 1).await?,
                EL2809Port::Pin9,
            )),
        ],
        Duration::from_millis(25),
        6,
    )];

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
