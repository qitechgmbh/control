use crate::{
    MACHINE_GLUETEX_V1, SerialDevice, SerialDeviceNew, SerialDeviceNewParams, VENDOR_QITECH,
    machine_identification::{
        DeviceHardwareIdentification, DeviceHardwareIdentificationSerial, DeviceIdentification,
        DeviceMachineIdentification, MachineIdentification, MachineIdentificationUnique,
    },
};
use control_core::helpers::hashing::{byte_folding_u16, hash_djb2};
use smol::lock::RwLock;
use std::sync::Arc;

/// Mock serial device for the Gluetex machine
/// This provides a minimal SerialDevice implementation that doesn't require actual hardware
#[derive(Debug)]
pub struct GluetexMockSerialDevice {
    pub path: String,
}

impl SerialDevice for GluetexMockSerialDevice {}

impl SerialDeviceNew for GluetexMockSerialDevice {
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Self>>), anyhow::Error>
    where
        Self: Sized,
    {
        let hash = hash_djb2(params.path.as_bytes());
        let serial = byte_folding_u16(&hash.to_le_bytes());

        let device_identification = DeviceIdentification {
            device_machine_identification: Some(DeviceMachineIdentification {
                machine_identification_unique: MachineIdentificationUnique {
                    machine_identification: MachineIdentification {
                        vendor: VENDOR_QITECH,
                        machine: MACHINE_GLUETEX_V1,
                    },
                    serial,
                },
                role: 0,
            }),
            device_hardware_identification: DeviceHardwareIdentification::Serial(
                DeviceHardwareIdentificationSerial {
                    path: params.path.clone(),
                },
            ),
        };

        let mock_serial_device = Arc::new(RwLock::new(GluetexMockSerialDevice {
            path: params.path.clone(),
        }));

        Ok((device_identification, mock_serial_device))
    }
}
