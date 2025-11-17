use futures::{FutureExt, select};
use machines::{
    AsyncThreadMessage, MachineConnection, MachineNewHardware, MachineNewHardwareSerial,
    MachineNewParams, SerialDevice, SerialDeviceIdentification, SerialDeviceNew,
    SerialDeviceNewParams,
    laser::LaserMachine,
    machine_identification::{
        DeviceIdentification, DeviceIdentificationIdentified, MachineIdentificationUnique,
    },
    registry::{MACHINE_REGISTRY, MachineRegistry},
    serial::{
        devices::laser::Laser,
        init::{SerialDetection, start_serial_discovery},
    },
    winder2::api::GenericEvent,
};

use app_state::{HotThreadMessage, SharedState};
use ethercat::{
    ethercat_discovery_info::{send_ethercat_discovering, send_ethercat_found},
    init::start_interface_discovery,
    setup::setup_loop,
};
use r#loop::start_loop_thread;

use panic::init_panic_handling;
use rest::init::start_api_thread;
use serialport::UsbPortInfo;
use smol::{
    channel::{Receiver, Sender},
    lock::RwLock,
};
use socketio::{main_namespace::machines_event::MachineObj, queue::start_socketio_queue};
use socketioxide::extract::SocketRef;
use std::{collections::HashMap, sync::Arc, time::Duration};

#[cfg(feature = "mock-machine")]
use mock_init::init_mock;

#[cfg(feature = "mock-machine")]
pub mod mock_init;

pub mod app_state;
pub mod ethercat;
pub mod logging;
pub mod r#loop;
pub mod panic;
pub mod performance_metrics;
pub mod rest;
pub mod socketio;

pub async fn send_empty_machines_event(shared_state: Arc<SharedState>) {
    shared_state.current_machines_meta.lock().await.clear();
    shared_state.clone().send_machines_event().await;
}

pub async fn add_serial_device(
    shared_state: Arc<SharedState>,
    device_identification: &DeviceIdentification,
    device: Arc<RwLock<dyn SerialDevice>>,
    machine_registry: &MachineRegistry,
    socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
) {
    tracing::info!("add_serial_device");
    let hardware = MachineNewHardwareSerial { device };

    let device_identification_identified: DeviceIdentificationIdentified = device_identification
        .clone()
        .try_into()
        .expect("Serial devices always have machine identification");

    let machine_identification: MachineIdentificationUnique = device_identification_identified
        .device_machine_identification
        .machine_identification_unique
        .clone();

    let new_machine = machine_registry.new_machine(&MachineNewParams {
        device_group: &vec![device_identification_identified.clone()],
        hardware: &MachineNewHardware::Serial(&hardware),
        socket_queue_tx,
        namespace: None,
        main_thread_channel: Some(shared_state.main_channel.clone()),
    });

    let machine = match new_machine {
        Ok(machine) => machine,
        Err(e) => {
            tracing::error!("{:?}", e);
            return;
        }
    };

    shared_state
        .add_machines_if_not_exists(vec![MachineObj {
            machine_identification_unique: machine_identification.clone(),
            error: None,
        }])
        .await;

    shared_state
        .api_machines
        .lock()
        .await
        .insert(machine_identification, machine.api_get_sender());

    let _ = shared_state
        .rt_machine_creation_channel
        .send(HotThreadMessage::AddMachines(vec![machine]))
        .await;
    shared_state.clone().send_machines_event().await;
}

async fn setup_ethercat(
    interface: &str,
    app_state: Arc<SharedState>,
    sender: Sender<HotThreadMessage>,
) {
    tracing::info!("Calling setup_loop");
    let res = setup_loop(&interface, app_state.clone()).await;
    match res {
        Ok(setup) => {
            let _ = sender.send(HotThreadMessage::AddEtherCatSetup(setup)).await;
            tracing::info!("Successfully initialized EtherCAT network");
        }

        Err(e) => {
            tracing::error!(
                "[{}::main] Failed to initialize EtherCAT network \n{:?}",
                module_path!(),
                e
            );
        }
    }
    send_ethercat_found(app_state.clone(), &interface).await;
}

pub async fn handle_serial_device_hotplug(
    app_state: Arc<SharedState>,
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
        let machines = app_state.current_machines_meta.lock().await;
        for machine in machines.iter() {
            if machine.machine_identification_unique.machine_identification
                == LaserMachine::MACHINE_IDENTIFICATION
            {
                unique_ident = Some(machine.machine_identification_unique.clone());
                break;
            }
        }
    }

    // Machine isnt connected, so add it back
    if laser.is_some() && unique_ident.is_none() {
        let serial_params = SerialDeviceNewParams {
            path: laser.unwrap(),
        };
        match Laser::new_serial(&serial_params) {
            Ok((device_identification, serial_device)) => {
                add_serial_device(
                    app_state.clone(),
                    &device_identification,
                    serial_device,
                    &MACHINE_REGISTRY,
                    app_state.socketio_setup.socket_queue_tx.clone(),
                )
                .await;
            }
            _ => (),
        };
    } else if laser.is_none() && unique_ident.is_some() {
        let unique_ident = unique_ident.unwrap();
        app_state
            .clone()
            .api_machines
            .lock()
            .await
            .remove(&unique_ident);
        app_state.clone().remove_machine(&unique_ident).await;

        let _ = app_state
            .clone()
            .rt_machine_creation_channel
            .send(HotThreadMessage::DeleteMachine(unique_ident))
            .await;

        app_state.clone().send_machines_event().await;
    }
}

async fn handle_async_requests(recv: Receiver<AsyncThreadMessage>, shared_state: Arc<SharedState>) {
    while let Ok(message) = recv.recv().await {
        match message {
            AsyncThreadMessage::NoMsg => (),
            AsyncThreadMessage::ConnectOneWayRequest(cross_connection) => {
                let api_machines_guard = shared_state.api_machines.lock().await;
                // The Src Connection is from the machine that recvs the request to connect
                // The Dest Connection the machine to which should be connected
                let src_ident = cross_connection.src;
                let dest_ident = cross_connection.dest;
                let src_sender = match api_machines_guard.get(&src_ident) {
                    Some(sender) => sender,
                    None => continue,
                };

                let dest_sender = match api_machines_guard.get(&dest_ident) {
                    Some(sender) => sender,
                    None => continue,
                };
                let connection = MachineConnection {
                    ident: dest_ident,
                    connection: dest_sender.clone(),
                };
                let res = src_sender
                    .send(machines::MachineMessage::ConnectToMachine(connection))
                    .await;
                match res {
                    Ok(_) => (),
                    Err(_) => tracing::error!("Failed to send MachineConnection"),
                }
            }
            AsyncThreadMessage::DisconnectMachines(cross_connection) => {
                let api_machines_guard = shared_state.api_machines.lock().await;
                // The Src Connection is from the machine that recvs the request to connect
                // The Dest Connection the machine to which should be connected
                let src_ident = cross_connection.src;
                let dest_ident = cross_connection.dest;
                let src_sender = match api_machines_guard.get(&src_ident) {
                    Some(sender) => sender,
                    None => continue,
                };

                let dest_sender = match api_machines_guard.get(&dest_ident) {
                    Some(sender) => sender,
                    None => continue,
                };

                let connection = MachineConnection {
                    ident: dest_ident.clone(),
                    connection: dest_sender.clone(),
                };

                let res = src_sender
                    .send(machines::MachineMessage::DisconnectMachine(connection))
                    .await;
                match res {
                    Ok(_) => (),
                    Err(e) => tracing::error!(
                        "AsyncThreadMessage::DisconnectMachines src:{:?} dest:{:?} error:{:?}",
                        src_ident,
                        dest_ident,
                        e
                    ),
                }
            }
        }
    }
}

#[cfg(feature = "development-build")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "development-build")]
fn setup_ctrlc_handler() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        eprintln!("Ctrl-C pressed, shutting down...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    running
}

#[cfg(feature = "heap-profile")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    logging::init_tracing();
    tracing::info!("Tracing initialized successfully");
    init_panic_handling();

    #[cfg(feature = "heap-profile")]
    let _profiler = dhat::Profiler::new_heap();

    #[cfg(feature = "development-build")]
    let running = setup_ctrlc_handler();

    const CYCLE_TARGET_TIME: Duration = Duration::from_micros(300);

    // for the "hot thread"
    let (sender, receiver) = smol::channel::unbounded();
    let (main_sender, main_receiver) = smol::channel::unbounded();
    let shared_state = SharedState::new(sender.clone(), main_sender.clone());
    let app_state = Arc::new(shared_state);
    let _loop_thread = start_loop_thread(receiver, CYCLE_TARGET_TIME);
    let _ = start_api_thread(app_state.clone());
    let mut socketio_fut = start_socketio_queue(app_state.clone()).fuse();

    #[cfg(not(feature = "mock-machine"))]
    let mut ethercat_fut = start_interface_discovery().fuse();

    let mut serial_fut = start_serial_discovery().fuse();
    let mut handle_async_machine_requests =
        smol::spawn(handle_async_requests(main_receiver, app_state.clone())).fuse();

    smol::block_on(async {
        send_empty_machines_event(app_state.clone()).await;
        send_ethercat_discovering(app_state.clone()).await;
    });

    #[cfg(feature = "mock-machine")]
    init_mock(app_state.clone()).expect("Failed to initialize mock machines");

    smol::block_on(async {
        loop {
            #[cfg(feature = "development-build")]
            if !running.load(Ordering::SeqCst) {
                tracing::info!("Shutdown signal received, exiting main loop.");
                break;
            }
            // lets the async runtime decide which future to run next
            #[cfg(not(feature = "mock-machine"))]
            select! {
                res = ethercat_fut => {
                    tracing::info!("EtherCAT task finished: {:?}", res);
                    match res {
                        Ok(interface) =>
                        {
                            setup_ethercat(&interface,app_state.clone(),sender.clone()).await;
                        },
                        Err(_) => (),
                    };

                },
                res = serial_fut => {
                    let _ = handle_serial_device_hotplug(app_state.clone(),res).await;
                    serial_fut = start_serial_discovery().fuse();
                },
                res = socketio_fut => {
                    // In theory it should never finish
                    tracing::warn!("SocketIO task finished: {:?}", res);
                },
                res = handle_async_machine_requests => {
                    tracing::warn!("Async handler task finished: {:?}", res);
                },
            }

            #[cfg(feature = "mock-machine")]
            select! {
                res = socketio_fut => {
                    // In theory it should never finish
                    tracing::warn!("SocketIO task finished: {:?}", res);
                },
                res = handle_async_machine_requests => {
                    tracing::warn!("Async handler task finished: {:?}", res);
                },
                _ = smol::Timer::after(Duration::from_millis(1)).fuse() => {

                }

            }
        }
    });
}
