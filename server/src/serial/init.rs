use crate::panic::send_panic;
use crate::socketio::main_namespace::MainNamespaceEvents;
use crate::socketio::main_namespace::machines_event::MachinesEventBuilder;
use crate::{app_state::AppState, machines::registry::MACHINE_REGISTRY};
use control_core::socketio::namespace::NamespaceCacheingLogic;
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
                                println!("{:?}", result);
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
                                drop(machine_guard);
                            }

                            // Process any pending socket connections for newly added serial machines.
                            // 
                            // Serial machines can be added/removed dynamically during runtime when USB devices
                            // are plugged/unplugged. If clients try to connect to a serial machine namespace
                            // before the device is detected and added, their connections are queued.
                            // This processes those queued connections once the serial machine becomes available.
                            {
                                let mut socketio_namespaces_guard = app_state_clone.socketio_setup.namespaces.write().await;
                                socketio_namespaces_guard.process_pending_connections(&app_state_clone).await;
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
