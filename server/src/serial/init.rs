use crate::panic::{PanicDetails, send_panic};
use crate::socketio::main_namespace::MainNamespaceEvents;
use crate::socketio::main_namespace::machines_event::MachinesEventBuilder;
use crate::{app_state::AppState, machines::registry::MACHINE_REGISTRY};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use smol::channel::Sender;
use std::{sync::Arc, thread, time::Duration};

pub fn init_serial(
    thread_panic_tx: Sender<PanicDetails>,
    app_state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
    let thread_panic_tx_clone = thread_panic_tx.clone();

    let app_state_clone = app_state.clone();

    thread::Builder::new()
        .name("SerialDetectionThread".to_owned())
        .spawn(move || {
            let app_state_clone = app_state_clone;
            smol::block_on(async move {
                send_panic(thread_panic_tx_clone);

                let rt = smol::LocalExecutor::new();
                rt.run(async {
                    loop {
                        let result = {
                            let mut serial_setup_guard = app_state_clone.serial_setup.write().await;

                            let mut port_result =
                                serial_setup_guard.serial_detection.check_ports().await;

                            let mut removed_signals = serial_setup_guard
                                .serial_detection
                                .check_remove_signals()
                                .await;

                            port_result.removed.append(&mut removed_signals);

                            port_result
                        };

                        if !result.added.is_empty() || !result.removed.is_empty() {
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

                            // Notify client via socketio
                            let app_state_event = app_state.clone();
                            let _ = smol::block_on(async {
                                let main_namespace = &mut app_state_event
                                    .socketio_setup
                                    .namespaces
                                    .write()
                                    .await
                                    .main_namespace;
                                let event =
                                    MachinesEventBuilder().build(app_state_event.clone()).await;
                                main_namespace
                                    .emit_cached(MainNamespaceEvents::MachinesEvent(event));
                            });
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
