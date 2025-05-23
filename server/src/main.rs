#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use app_state::AppState;
use std::{panic::catch_unwind, process::exit, sync::Arc};

use env_logger::Env;
use ethercat::init::init_ethercat;
use r#loop::init_loop;
use rest::init::init_api;
use serial::init::init_serial;
use smol::channel::unbounded;

pub mod app_state;
pub mod ethercat;
pub mod r#loop;
pub mod machines;
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
    #[cfg(feature = "dhat-heap")]
    let profiler = dhat::Profiler::new_heap();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let app_state = Arc::new(AppState::new());

    let (thread_panic_tx, thread_panic_rx) = unbounded::<&'static str>();

    init_serial(thread_panic_tx.clone(), app_state.clone()).expect("Failed to initialize Serial");
    init_api(thread_panic_tx.clone(), app_state.clone()).expect("Failed to initialize API");
    init_ethercat(thread_panic_tx.clone(), app_state.clone())
        .expect("Failed to initialize EtherCAT");
    init_loop(thread_panic_tx, app_state).expect("Failed to initialize loop");

    smol::block_on(async {
        #[cfg(feature = "dhat-heap")]
        let dhat_analysis_time = std::time::Duration::from_secs(60);
        #[cfg(feature = "dhat-heap")]
        let dhat_analysis_start = std::time::Instant::now();

        loop {
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
