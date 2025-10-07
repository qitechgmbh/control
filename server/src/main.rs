#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use app_state::AppState;
use r#loop::init_loop;
#[cfg(feature = "mock-machine")]
use mock::init::init_mock;
use rest::init::init_api;
#[cfg(not(feature = "mock-machine"))]
use serial::init::init_serial;
use std::sync::Arc;

use crate::panic::init_panic;
use crate::socketio::queue::init_socketio_queue;

pub mod app_state;
pub mod ethercat;
pub mod logging;
pub mod r#loop;
pub mod machines;
#[cfg(feature = "mock-machine")]
pub mod mock;
pub mod panic;
pub mod performance_metrics;
pub mod rest;
pub mod serial;
pub mod socketio;

use crate::ethercat::init::init_ethercat;

fn main() {
    // Initialize panic handling
    let thread_panic_tx = init_panic();
    logging::init_tracing();
    tracing::info!("Tracing initialized successfully");
    if let Err(e) = control_core::realtime::lock_memory() {
        tracing::error!("[{}::main] Failed to lock memory: {:?}", module_path!(), e);
    } else {
        tracing::info!("[{}::main] Memory locked successfully", module_path!());
    }
    let app_state = Arc::new(AppState::new());

    // Spawn init thread
    let init_thread = std::thread::Builder::new()
        .name("init".to_string())
        .spawn({
            let thread_panic_tx = thread_panic_tx;
            let app_state = app_state;
            move || {
                #[cfg(feature = "dhat-heap")]
                init_dhat_heap_profiling();

                init_socketio_queue(thread_panic_tx.clone(), app_state.clone());
                init_api(thread_panic_tx.clone(), app_state.clone())
                    .expect("Failed to initialize API");
                init_loop(thread_panic_tx.clone(), app_state.clone())
                    .expect("Failed to initialize loop");

                #[cfg(feature = "mock-machine")]
                init_mock(app_state.clone()).expect("Failed to initialize mock machines");

                #[cfg(not(feature = "mock-machine"))]
                init_serial(thread_panic_tx.clone(), app_state.clone())
                    .expect("Failed to initialize Serial");

                #[cfg(not(feature = "mock-machine"))]
                init_ethercat(thread_panic_tx.clone(), app_state)
                    .expect("Failed to initialize EtherCAT");
            }
        })
        .expect("Failed to spawn init thread");

    // Wait for init thread to complete
    init_thread.join().expect("Init thread panicked");

    // Keep the main thread alive indefinitely
    // The program should only exit via panic handling or external signals
    loop {
        // better way to "sleep", uses basically
        std::thread::park();
    }
}

#[cfg(feature = "dhat-heap")]
fn init_dhat_heap_profiling() {
    use std::{process::exit, time::Duration};

    let profiler = dhat::Profiler::new_heap();

    smol::block_on(async {
        let dhat_analysis_time = std::time::Duration::from_secs(60);
        let dhat_analysis_start = std::time::Instant::now();
        tracing::info!(
            "Starting dhat heap profiler for {} seconds",
            dhat_analysis_time.as_secs()
        );

        loop {
            use std::time::Duration;
            smol::Timer::after(Duration::from_secs(1)).await;

            // if `dhat-heap` is enabled, we will analyze the heap for 60 seconds and then exit
            if dhat_analysis_start.elapsed() > dhat_analysis_time {
                drop(profiler);
                exit(0)
            }
        }
    })
}
