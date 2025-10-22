use anyhow::Result;
use control_core::{
    machines::identification::MachineIdentificationUnique,
    serial::{
        SerialDeviceIdentification, SerialDeviceNew, SerialDeviceNewParams,
        serial_detection::SerialDetection,
    },
    socketio::namespace::NamespaceCacheingLogic,
};
use futures::{FutureExt, select};
use machines::{laser::LaserMachine, registry::MACHINE_REGISTRY};
#[cfg(feature = "mock-machine")]
use mock::init::init_mock;
use serialport::UsbPortInfo;

#[cfg(feature = "mock-machine")]
pub mod mock;

use crate::panic::init_panic_handling;
use app_state::AppState;
use ethercat::init::start_interface_discovery;
use r#loop::start_loop_thread;
use rest::init::start_api_thread;
use serial::{
    devices::laser::Laser, init::start_serial_discovery, registry::SERIAL_DEVICE_REGISTRY,
};
use socketio::{
    main_namespace::{MainNamespaceEvents, machines_event::MachinesEventBuilder},
    queue::start_socketio_queue,
};
use std::{collections::HashMap, sync::Arc};
pub mod app_state;
pub mod ethercat;
pub mod logging;
pub mod r#loop;
pub mod machines;
pub mod panic;
pub mod performance_metrics;
pub mod rest;
pub mod serial;
pub mod socketio;

// This function handles a disconnection and or reconnection of Laser
// You can easily rewrite it when we have more serialdevices
// TODO: Add A static list of known SeriadeviceIdents
async fn handle_serial_device_hotplug(
    app_state: Arc<AppState>,
    map: Option<HashMap<String, UsbPortInfo>>,
) {
    let laser_ident = SerialDeviceIdentification {
        vendor_id: 0x0403,
        product_id: 0x6001,
    };

    let laser = match map {
        Some(map) => SerialDetection::get_path_by_id(laser_ident, map),
        None => None,
    };

    let mut unique_ident: Option<MachineIdentificationUnique> = None;

    {
        let machines = app_state.machines.read_arc_blocking();
        for (id, _) in machines.iter() {
            if id.machine_identification == LaserMachine::MACHINE_IDENTIFICATION {
                unique_ident = Some(id.clone());
                break;
            }
        }
    }

    // Machine isnt connected, so add it back
    if laser.is_some() && unique_ident.is_none() {
        let serial_params = SerialDeviceNewParams {
            path: laser.unwrap(),
        };
        let _ = match Laser::new_serial(&serial_params) {
            Ok((device_identification, serial_device)) => {
                {
                    let mut machines = app_state.machines.write().await;
                    machines.add_serial_device(
                        &device_identification,
                        serial_device,
                        &MACHINE_REGISTRY,
                        app_state.socketio_setup.socket_queue_tx.clone(),
                        Arc::downgrade(&app_state.machines),
                    );
                }

                let app_state_event = app_state.clone();
                let main_namespace = &mut app_state_event
                    .socketio_setup
                    .namespaces
                    .write()
                    .await
                    .main_namespace;
                let event = MachinesEventBuilder().build(app_state_event.clone());
                main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
            }
            _ => (),
        };
    } else if laser.is_none() && unique_ident.is_some() {
        tracing::info!("hello non existing laser");
        let serial_params = SerialDeviceNewParams {
            path: "".to_string(),
        };

        let _ = match Laser::new_serial(&serial_params) {
            Ok((device_identification, _)) => {
                app_state
                    .machines
                    .write()
                    .await
                    .remove_serial_device(&device_identification);
                drop(app_state.machines.clone());

                app_state
                    .machines
                    .write_arc()
                    .await
                    .serial_machines
                    .remove(&unique_ident.clone().unwrap());

                app_state
                    .machines
                    .write_arc()
                    .await
                    .ethercat_machines
                    .remove(&unique_ident.clone().unwrap());

                drop(app_state.machines.clone());

                let main_namespace = &mut app_state
                    .socketio_setup
                    .namespaces
                    .write()
                    .await
                    .main_namespace;

                tracing::info!("{:?}", app_state.machines);
                let event = MachinesEventBuilder().build(app_state.clone());
                main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
            }
            _ => (),
        };
    }
}

fn main() {
    logging::init_tracing();
    tracing::info!("Tracing initialized successfully");
    init_panic_handling();

    let app_state = Arc::new(AppState::new());

    let _ = start_api_thread(app_state.clone());
    let _ = start_loop_thread(app_state.clone());

    let mut socketio_fut = start_socketio_queue(app_state.clone()).fuse();
    let mut ethercat_fut = start_interface_discovery().fuse();
    let mut serial_fut = start_serial_discovery().fuse();

    smol::block_on(async {
        loop {
            // lets the async runtime decide which future to run next
            select! {
                res = ethercat_fut => {
                    // Should only complete once
                    tracing::info!("EtherCAT task finished: {:?}", res);
                },
                res = serial_fut => {
                  //  tracing::info!("Serial discovery finished: {:?}", res);
                    let _ = handle_serial_device_hotplug(app_state.clone(),res).await;
                    serial_fut = start_serial_discovery().fuse();
                },
                res = socketio_fut => {
                    // In theory it should never finish
                    tracing::info!("SocketIO task finished: {:?}", res);
                },
                _ = smol::Timer::after(std::time::Duration::from_millis(250)).fuse() => {} // Nothing to do just sleep
            }
        }
    });
}
