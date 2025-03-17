use super::r#loop::setup_loop;
use crate::app_state::AppState;
use std::sync::Arc;
use thread_priority::{ThreadBuilderExt, ThreadPriority};

pub fn init_ethercat(app_state: Arc<AppState>) {
    let interface = "en6";

    tokio::spawn(async move {
        std::thread::Builder::new()
            .name("EthercatThread".to_owned())
            .spawn_with_priority(ThreadPriority::Max, move |_| {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                runtime.block_on(async {
                    log::info!("Starting Ethercat PDU loop");
                    let result = setup_loop(interface, app_state.clone()).await;
                    if let Err(e) = result {
                        panic!("Failed to setup Ethercat: {:?}", e);
                    } else {
                        log::info!("Ethercat loop exited");
                    }
                })
            })
            .unwrap();
    });
}
