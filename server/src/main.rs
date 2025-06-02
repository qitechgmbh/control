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

use ethercat::init::init_ethercat;
use r#loop::init_loop;
use rest::init::init_api;
#[cfg(not(feature = "mock-machine"))]
use serial::init::init_serial;
use smol::channel::unbounded;

#[cfg(all(not(target_env = "msvc"), not(feature = "dhat-heap")))]
use jemalloc_stats::init_jemalloc_stats;

pub mod app_state;
pub mod ethercat;
pub mod logging;
pub mod r#loop;
pub mod machines;
#[cfg(feature = "mock-machine")]
pub mod mock;
pub mod panic;
pub mod rest;
pub mod serial;
pub mod socketio;

#[cfg(all(not(target_env = "msvc"), not(feature = "dhat-heap")))]
pub mod jemalloc_stats;

fn main() {
    logging::init_tracing();
    tracing::info!("Tracing initialized successfully");

    #[cfg(all(not(target_env = "msvc"), not(feature = "dhat-heap")))]
    init_jemalloc_stats();

    // if the program panics we restart all of it
    match catch_unwind(|| main2()) {
        Ok(_) => {
            tracing::info!("Program ended normally");
            #[cfg(feature = "tracing-otel")]
            exit(0);
        }
        Err(err) => {
            tracing::error!("Program panicked: {:?}", err);
            #[cfg(feature = "tracing-otel")]
            exit(1);
        }
    }
}

fn main2() {
    let app_state = Arc::new(AppState::new());

    let (thread_panic_tx, thread_panic_rx) = unbounded::<&'static str>();

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
        tracing::info!(
            "Starting dhat heap profiler for {} seconds",
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
                    ::tracing::error!("Thread panicked: {:?}", panic_details);
                    exit(1);
                }
                Err(_) => {}
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        // setup logging
        logging::init_tracing();
        ::tracing::info!("Running main test");
    }
}
