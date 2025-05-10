use std::{
    sync::Arc,
    thread,
    time::Duration,
};
use smol::channel::Sender;
use crate::panic::PanicDetails;
use crate::app_state::AppState;
use crate::panic::send_panic;

pub fn init_serial(
    thread_panic_tx: Sender<PanicDetails>,
    app_state: Arc<AppState>
) -> Result<(), anyhow::Error> {
    
    let thread_panic_tx_clone = thread_panic_tx.clone();

    let app_state_clone = app_state.clone();
    thread::Builder::new()
        .name("SerialTxRxThread".to_owned())
        .spawn(move || {
            let app_state_clone = app_state_clone;
            smol::block_on(async move {
                send_panic("SerialThread", thread_panic_tx_clone);
                
                let rt = smol::LocalExecutor::new();
                rt.run(async {
                    loop {
                        { 
                            app_state_clone.serial_setup.write().await.cycle().await;
                        }
                        // Sleep for a while to avoid excessive CPU usage
                        thread::sleep(Duration::from_millis(300));
                    }
                })
                .await;
            });
        })
        .expect("Failed to spawn SerialTxRxThread");
    Ok(())
}
