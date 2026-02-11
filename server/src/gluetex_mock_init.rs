use crate::add_serial_device;
use crate::app_state::SharedState;
use machines::registry::MACHINE_REGISTRY;
use machines::{
    SerialDeviceNew, SerialDeviceNewParams, serial::devices::gluetex_mock::GluetexMockSerialDevice,
};
use std::sync::Arc;

pub fn init_gluetex_mock(app_state: Arc<SharedState>) -> Result<(), anyhow::Error> {
    smol::block_on(async {
        let serial_params = SerialDeviceNewParams {
            path: "/dev/mock-gluetex-serial".to_string(),
        };

        match GluetexMockSerialDevice::new_serial(&serial_params) {
            Ok((device_identification, mock_serial_device)) => {
                add_serial_device(
                    app_state.clone(),
                    &device_identification,
                    mock_serial_device,
                    &MACHINE_REGISTRY,
                    app_state.socketio_setup.socket_queue_tx.clone(),
                )
                .await;
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to create gluetex mock serial device: {}", e);
                Err(e)
            }
        }
    })
}
