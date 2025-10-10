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

#[cfg(feature = "laser-mock")]
use {
    crate::app_state::AppState,
    crate::machines::registry::MACHINE_REGISTRY,
    crate::serial::devices::mock_laser::MockLaserDevice,
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
        let (device_thread_panic_tx, _device_thread_panic_rx) = smol::channel::unbounded();
        let serial_params = SerialDeviceNewParams {
            path: "/dev/mock-serial".to_string(),
            device_thread_panic_tx,
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

#[cfg(feature = "laser-mock")]
pub fn init_laser_mock(app_state: Arc<AppState>) -> Result<(), anyhow::Error> {
    // For mock laser device, we need to manually create and add it to the machine manager
    // since it won't be detected by the serial detection loop
    return smol::block_on(async {
        // Create a mock laser device manually
        let (device_thread_panic_tx, _device_thread_panic_rx) = smol::channel::unbounded();
        let serial_params = SerialDeviceNewParams {
            path: "/dev/mock-laser".to_string(),
            device_thread_panic_tx: device_thread_panic_tx.clone(),
        };

        // Create the mock laser device
        match MockLaserDevice::new_serial(&serial_params) {
            Ok((device_identification, mock_laser_device)) => {
                // Clone the device for the background task before moving it
                let mock_laser_clone = mock_laser_device.clone();

                // Add the mock device to the machine manager
                {
                    let mut machine_guard = app_state.machines.write().await;
                    machine_guard.add_serial_device(
                        &device_identification,
                        mock_laser_device,
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
                let event = MachinesEventBuilder().build(app_state_event.clone()).await;
                main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));

                // Spawn background task to update laser data periodically
                smol::spawn(async move {
                    loop {
                        // Update data approximately 30 times per second
                        smol::Timer::after(std::time::Duration::from_millis(33)).await;
                        let mut laser = mock_laser_clone.write().await;
                        laser.update_data();
                    }
                })
                .detach();

                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to create mock laser device: {}", e);
                return Err(e);
            }
        }
    });
}
