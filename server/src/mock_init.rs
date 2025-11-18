use crate::add_serial_device;
use crate::app_state::SharedState;
use crate::socketio::main_namespace::MainNamespaceEvents;
use crate::socketio::main_namespace::machines_event::MachinesEventBuilder;
use machines::registry::MACHINE_REGISTRY;
use std::sync::Arc;

pub fn init_mock(app_state: Arc<SharedState>) -> Result<(), anyhow::Error> {
    // For mock devices, we need to manually create and add them to the machine manager
    // since they won't be detected by the serial detection loop
    return smol::block_on(async {
        // Create a mock serial device manually

        use machines::{
            SerialDeviceNew, SerialDeviceNewParams,
            serial::devices::{
                extruder_mock::ExtruderMockSerialDevice, mock::MockSerialDevice,
                winder_mock::WinderMockSerialDevice,
            },
        };
        let serial_params = SerialDeviceNewParams {
            path: "/dev/mock-serial".to_string(),
        };

        // Create the mock serial device
        let _ = match MockSerialDevice::new_serial(&serial_params) {
            Ok((device_identification, mock_serial_device)) => {
                // Add the mock device to the machine manager
                {
                    use crate::add_serial_device;
                    use machines::registry::MACHINE_REGISTRY;
                    add_serial_device(
                        app_state.clone(),
                        &device_identification,
                        mock_serial_device,
                        &MACHINE_REGISTRY,
                        app_state.socketio_setup.socket_queue_tx.clone(),
                    )
                    .await;
                }
                Ok::<(), anyhow::Error>(())
            }
            Err(e) => {
                tracing::error!("Failed to create mock serial device: {}", e);
                return Err(e);
            }
        };

        let _ = match ExtruderMockSerialDevice::new_serial(&serial_params) {
            Ok((device_identification, mock_serial_device)) => {
                // Add the mock device to the machine manager
                {
                    use crate::add_serial_device;
                    use machines::registry::MACHINE_REGISTRY;

                    add_serial_device(
                        app_state.clone(),
                        &device_identification,
                        mock_serial_device,
                        &MACHINE_REGISTRY,
                        app_state.socketio_setup.socket_queue_tx.clone(),
                    )
                    .await;
                }

                Ok::<(), anyhow::Error>(())
            }
            Err(e) => {
                tracing::error!("Failed to create extruder mock serial device: {}", e);
                return Err(e);
            }
        };

        match WinderMockSerialDevice::new_serial(&serial_params) {
            Ok((device_identification, mock_serial_device)) => {
                // Add the mock device to the machine manager
                add_serial_device(
                    app_state.clone(),
                    &device_identification,
                    mock_serial_device,
                    &MACHINE_REGISTRY,
                    app_state.clone().socketio_setup.socket_queue_tx.clone(),
                )
                .await;

                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to create extruder mock serial device: {}", e);
                return Err(e);
            }
        }
    });
}
