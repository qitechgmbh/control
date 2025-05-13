use crate::panic::send_panic;
use crate::{app_state::AppState, machines::registry::MACHINE_REGISTRY};
use smol::channel::Sender;
use std::{sync::Arc, thread, time::Duration};

pub fn init_serial(
    thread_panic_tx: Sender<&'static str>,
    app_state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
    let thread_panic_tx_clone = thread_panic_tx.clone();

    let app_state_clone = app_state.clone();
    thread::Builder::new()
        .name("SerialDetectionThread".to_owned())
        .spawn(move || {
            let app_state_clone = app_state_clone;
            smol::block_on(async move {
                send_panic("SerialDetectionThread", thread_panic_tx_clone);

                let rt = smol::LocalExecutor::new();
                rt.run(async {
                    loop {
                        let result = {
                            let mut serial_setup_guard = app_state_clone.serial_setup.write().await;
                            serial_setup_guard
                                .serial_detection
                                .check_remove_signals()
                                .await;
                            serial_setup_guard.serial_detection.check_ports().await
                        };


                        // sync serial device discovery to machine manager
                        {
                            let mut machine_guard = app_state_clone.machines.write().await;

                            // turn added devices into machines
                            for (device_identifiaction, device) in result.added {
                                machine_guard.add_serial_device(
                                    &device_identifiaction,
                                    device.clone(),
                                    &MACHINE_REGISTRY,
                                )
                            }
                            for device_identification in result.removed {
                                machine_guard.remove_serial_device(&device_identification)
                            }
                        }

                        smol::Timer::after(Duration::from_millis(300)).await;
                    }
                })
                .await;
            });
        })
        .expect("Failed to spawn SerialTxRxThread");
    Ok(())
}
