#[cfg(feature = "mock-machine")]
use crate::serial::devices::extruder_mock::ExtruderMockSerialDevice;
#[cfg(feature = "mock-machine")]
use {
    crate::app_state::AppState,
    crate::machines::registry::MACHINE_REGISTRY,
    crate::serial::devices::mock::MockSerialDevice,
    crate::socketio::main_namespace::{MainNamespaceEvents, machines_event::MachinesEventBuilder},
    control_core::{
        serial::{SerialDeviceNew, SerialDeviceNewParams},
        socketio::namespace::NamespaceCacheingLogic,
    },
    std::sync::Arc,
};
#[cfg(feature = "mock-machine")]
pub fn init_mock(app_state: Arc<AppState>) -> Result<(), anyhow::Error> {
    // For mock devices, we need to manually create and add them to the machine manager
    // since they won't be detected by the serial detection loop
    return smol::block_on(async {
        // Create a mock serial device manually
        let serial_params = SerialDeviceNewParams {
            path: "/dev/mock-serial".to_string(),
        };

        // Create the mock serial device
        let _ = match MockSerialDevice::new_serial(&serial_params) {
            Ok((device_identification, mock_serial_device)) => {
                // Add the mock device to the machine manager
                {
                    let mut machine_guard = app_state.machines.write().await;
                    machine_guard.add_serial_device(
                        &device_identification,
                        mock_serial_device,
                        &MACHINE_REGISTRY,
                        app_state.socketio_setup.socket_queue_tx.clone(),
                        Arc::downgrade(&app_state.machines),
                    );
                }

                // Notify clients via socketio about the new machine
                let app_state_event = app_state.clone();
                let main_namespace = &mut app_state_event
                    .socketio_setup
                    .namespaces
                    .write()
                    .await
                    .main_namespace;
                let event = MachinesEventBuilder().build(app_state_event.clone());
                main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
                Ok::<(), anyhow::Error>(())
            }
            Err(e) => {
                tracing::error!("Failed to create mock serial device: {}", e);
                return Err(e);
            }
        };

        match ExtruderMockSerialDevice::new_serial(&serial_params) {
            Ok((device_identification, mock_serial_device)) => {
                // Add the mock device to the machine manager
                {
                    let mut machine_guard = app_state.machines.write().await;
                    machine_guard.add_serial_device(
                        &device_identification,
                        mock_serial_device,
                        &MACHINE_REGISTRY,
                        app_state.socketio_setup.socket_queue_tx.clone(),
                        Arc::downgrade(&app_state.machines),
                    );
                }

                // Notify clients via socketio about the new machine
                let app_state_event = app_state.clone();
                let main_namespace = &mut app_state_event
                    .socketio_setup
                    .namespaces
                    .write()
                    .await
                    .main_namespace;
                let event = MachinesEventBuilder().build(app_state_event.clone());
                main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to create extruder mock serial device: {}", e);
                return Err(e);
            }
        }
    });
}
