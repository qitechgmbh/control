use futures::{FutureExt, select};
#[cfg(feature = "mock-machine")]
use mock::init::init_mock;

#[cfg(feature = "mock-machine")]
pub mod mock;

use crate::{ethercat::setup::setup_loop, panic::init_panic_handling};
use app_state::AppState;
use ethercat::{
    ethercat_discovery_info::{send_ethercat_discovering, send_ethercat_found},
    init::start_interface_discovery,
};
use r#loop::start_loop_thread;
use rest::init::start_api_thread;
use serial::init::{handle_serial_device_hotplug, start_serial_discovery};
use socketio::queue::start_socketio_queue;
use std::sync::Arc;
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

fn main() {
    logging::init_tracing();
    tracing::info!("Tracing initialized successfully");
    init_panic_handling();

    let app_state = Arc::new(AppState::new());

    #[cfg(feature = "mock-machine")]
    init_mock(app_state.clone()).expect("Failed to initialize mock machines");

    let _ = start_api_thread(app_state.clone());
    let _ = start_loop_thread(app_state.clone());
    let mut socketio_fut = start_socketio_queue(app_state.clone()).fuse();
    let mut ethercat_fut = start_interface_discovery().fuse();
    let mut serial_fut = start_serial_discovery().fuse();

    smol::block_on(async { send_ethercat_discovering(app_state.clone()).await });

    smol::block_on(async {
        loop {

            // lets the async runtime decide which future to run next
            select! {
                res = ethercat_fut => {
                    tracing::info!("EtherCAT task finished: {:?}", res);
                    match res {
                        Ok(interface) =>
                        {
                            tracing::info!("Calling setup_loop");
                            let res = setup_loop(&interface, app_state.clone()).await;
                            match res {
                                Ok(_) => tracing::info!("Successfully initialized EtherCAT network"),
                                Err(e) => {
                                    tracing::error!(
                                        "[{}::init_loop] Failed to initialize EtherCAT network \n{:?}",
                                        module_path!(),
                                        e
                                    );
                                }
                            }
                            send_ethercat_found(app_state.clone(), &interface).await;
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
            }
        }
    });
}
