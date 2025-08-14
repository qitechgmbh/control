use std::sync::Arc;

use control_core::{
    helpers::hashing::{byte_folding_u16, hash_djb2},
    machines::identification::{
        DeviceHardwareIdentification, DeviceHardwareIdentificationSerial, DeviceIdentification,
        DeviceMachineIdentification, MachineIdentification, MachineIdentificationUnique,
    },
    serial::{SerialDevice, SerialDeviceNew, SerialDeviceNewParams},
};
use smol::lock::RwLock;

use crate::machines::{MACHINE_MOCK, VENDOR_QITECH};

/// Mock serial device for testing MockMachine
/// This provides a minimal SerialDevice implementation that doesn't require actual hardware
#[derive(Debug)]
pub struct MockSerialDevice {
    pub path: String,
}

impl SerialDevice for MockSerialDevice {}

impl SerialDeviceNew for MockSerialDevice {
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Self>>), anyhow::Error>
    where
        Self: Sized,
    {
        // Generate a unique serial number based on the path
        let hash = hash_djb2(params.path.as_bytes());
        let serial = byte_folding_u16(&hash.to_le_bytes());

        let device_identification = DeviceIdentification {
            device_machine_identification: Some(DeviceMachineIdentification {
                machine_identification_unique: MachineIdentificationUnique {
                    machine_identification: MachineIdentification {
                        vendor: VENDOR_QITECH,
                        machine: MACHINE_MOCK,
                    },
                    serial: serial,
                },
                role: 0,
            }),
            device_hardware_identification: DeviceHardwareIdentification::Serial(
                DeviceHardwareIdentificationSerial {
                    path: params.path.clone(),
                },
            ),
        };

        let mock_serial_device = Arc::new(RwLock::new(MockSerialDevice {
            path: params.path.clone(),
        }));

        Ok((device_identification, mock_serial_device))
    }
}
