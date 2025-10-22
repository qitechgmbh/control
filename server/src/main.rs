#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use app_state::AppState;
use ethercat::init::start_interface_discovery;
use r#loop::init_loop;
#[cfg(feature = "mock-machine")]
use mock::init::init_mock;
use rest::init::init_api;
#[cfg(not(feature = "mock-machine"))]
use serial::init::init_serial;
use std::sync::Arc;

use crate::panic::init_panic_handling;
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

fn main() {
    logging::init_tracing();
    tracing::info!("Tracing initialized successfully");
    init_panic_handling();

    let ethercat_interface: Option<String> = None;
    let ethercat_interface_found_fut = start_interface_discovery();

    let app_state = Arc::new(AppState::new());

    // init_socketio_queue(app_state.clone());
    // init_api(app_state.clone()).expect("Failed to initialize API");
    // init_loop(app_state.clone()).expect("Failed to initialize loop");

    // #[cfg(not(feature = "mock-machine"))]
    init_serial(app_state.clone()).expect("Failed to initialize Serial");

    // Keep the main thread alive indefinitely
    // The program should only exit via panic handling or external signals
    loop {
        // better way to "sleep"
        std::thread::park();
    }
}
