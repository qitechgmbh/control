use std::sync::Arc;

use thread_priority::{ThreadBuilderExt, ThreadPriority};

use crate::app_state::AppState;

use super::r#loop::setup_loop;

pub fn init_ethercat(app_state: Arc<AppState>) {
    let interface = "en10";

    tokio::spawn(async move {
        std::thread::Builder::new()
            .name("MyNewThread".to_owned())
            .spawn_with_priority(ThreadPriority::Max, move |_| {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                runtime.block_on(async {
                    log::info!("Starting Ethercat PDU loop");
                    setup_loop(interface, app_state.clone()).await
                })
            })
            .unwrap();
    });
}
