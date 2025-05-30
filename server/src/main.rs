#[cfg(all(not(target_env = "msvc"), not(feature = "dhat-heap")))]
use tikv_jemallocator::Jemalloc;

#[cfg(all(not(target_env = "msvc"), not(feature = "dhat-heap")))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use app_state::AppState;
use control_core::helpers::loop_trottle::LoopThrottle;
#[cfg(feature = "mock-machine")]
use mock::init::init_mock;
use std::{panic::catch_unwind, process::exit, sync::Arc, time::Duration};

use env_logger::Env;
use ethercat::init::init_ethercat;
use r#loop::init_loop;
use rest::init::init_api;
#[cfg(not(feature = "mock-machine"))]
use serial::init::init_serial;
use smol::{Timer, channel::unbounded};

pub mod app_state;
pub mod ethercat;
pub mod r#loop;
pub mod machines;
#[cfg(feature = "mock-machine")]
pub mod mock;
pub mod panic;
pub mod rest;
pub mod serial;
pub mod socketio;

fn main() {
    // if the program panics we restart all of it
    match catch_unwind(|| main2()) {
        Ok(_) => {
            log::info!("[{}::main] Program ended normally", module_path!());
            exit(0);
        }
        Err(err) => {
            log::error!("[{}::main] Program panicked: {:?}", module_path!(), err);
            exit(1);
        }
    }
}

fn main2() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let app_state = Arc::new(AppState::new());

    let (thread_panic_tx, thread_panic_rx) = unbounded::<&'static str>();

    init_debug(app_state.clone());
    init_api(thread_panic_tx.clone(), app_state.clone()).expect("Failed to initialize API");
    #[cfg(not(feature = "mock-machine"))]
    init_serial(thread_panic_tx.clone(), app_state.clone()).expect("Failed to initialize Serial");
    #[cfg(not(feature = "mock-machine"))]
    init_ethercat(thread_panic_tx.clone(), app_state.clone())
        .expect("Failed to initialize EtherCAT");
    #[cfg(feature = "mock-machine")]
    init_mock(app_state.clone()).expect("Failed to initialize mock machines");
    init_loop(thread_panic_tx, app_state).expect("Failed to initialize loop");

    #[cfg(feature = "dhat-heap")]
    let profiler = dhat::Profiler::new_heap();

    smol::block_on(async {
        #[cfg(feature = "dhat-heap")]
        let dhat_analysis_time = std::time::Duration::from_secs(60);
        #[cfg(feature = "dhat-heap")]
        let dhat_analysis_start = std::time::Instant::now();
        #[cfg(feature = "dhat-heap")]
        log::info!(
            "[{}::main] Starting dhat heap profiler for {} seconds",
            module_path!(),
            dhat_analysis_time.as_secs()
        );

        let mut throttle = LoopThrottle::new(Duration::from_millis(100), 10, None);

        loop {
            // Without throttleing the loop it would run as fast as possible, consuming 100% CPU
            // We throttle it to 100ms, which is a good balance between responsiveness and CPU usage
            throttle.sleep().await;

            // if `dhat-heap` is enabled, we will analyze the heap for 60 seconds and then exit
            #[cfg(feature = "dhat-heap")]
            if dhat_analysis_start.elapsed() > dhat_analysis_time {
                drop(profiler);
                exit(0)
            }

            match thread_panic_rx.try_recv() {
                Ok(panic_details) => {
                    log::error!(
                        "[{}::main] Thread panicked: {:?}",
                        module_path!(),
                        panic_details
                    );
                    exit(1);
                }
                Err(_) => {}
            }
        }
    })
}

/// Starts a thread that rust debug code every second.
fn init_debug(app_state: Arc<AppState>) {
    std::thread::Builder::new()
        .name("debug".to_string())
        .spawn(move || {
            log::info!("[{}::debug] Debug thread running", module_path!());
            smol::block_on(async move {
                let mut throttle = LoopThrottle::new(Duration::from_millis(10), 10, None);
                loop {
                    throttle.sleep().await;

                    // iterate all machines
                    let machines_guard = app_state.machines.read().await;
                    for (midu, machine) in machines_guard.iter() {
                        // log the machine name and its state
                        log::info!("[{}::debug] Machine: {:?}", module_path!(), midu);

                        // get machine
                        let mut machine = match machine {
                            Ok(machine) => machine.lock().await,
                            Err(_) => continue,
                        };

                        let namespace = machine.api_event_namespace();

                        // Log cached events
                        for (event_name, events) in namespace.events.iter() {
                            // log the event name and its length
                            log::info!(
                                "[{}::debug] Event: {}, Amount: {}",
                                module_path!(),
                                event_name,
                                events.len()
                            );
                        }

                        // Log socket queues
                        for (socket_name, queue) in namespace.socket_queues.iter() {
                            // log the socket name and its length
                            let queue_lock = loop {
                                match queue.queue.lock() {
                                    Ok(queue_lock) => break queue_lock,
                                    Err(_) => {
                                        Timer::after(Duration::from_millis(1)).await;
                                        continue;
                                    }
                                }
                            };

                            log::info!(
                                "[{}::debug] Socket: {}, Amount: {}",
                                module_path!(),
                                socket_name,
                                queue_lock.len()
                            );
                        }
                    }
                }
            })
        })
        .expect("Failed to spawn debug thread");
}
