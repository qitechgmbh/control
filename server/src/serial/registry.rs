use control_core::serial::{SerialDeviceIdentification, registry::SerialDeviceRegistry};
use lazy_static::lazy_static;

use crate::serial::devices::laser::Laser;

#[cfg(feature = "mock-machine")]
use crate::serial::devices::mock::MockSerialDevice;

#[cfg(feature = "laser-mock")]
use crate::serial::devices::mock_laser::MockLaserDevice;

lazy_static! {
    pub static ref SERIAL_DEVICE_REGISTRY: SerialDeviceRegistry = {
        let mut sdr = SerialDeviceRegistry::new();
        sdr.register::<Laser>(SerialDeviceIdentification {
            vendor_id: 0x0403,
            product_id: 0x6001,
        });

        // Register MockSerialDevice when mock-machine feature is enabled
        #[cfg(feature = "mock-machine")]
        sdr.register::<MockSerialDevice>(SerialDeviceIdentification {
            vendor_id: 0x0001, // VENDOR_QITECH
            product_id: 0x0007, // MACHINE_MOCK
        });

        // Register MockLaserDevice when laser-mock feature is enabled
        #[cfg(feature = "laser-mock")]
        sdr.register::<MockLaserDevice>(SerialDeviceIdentification {
            vendor_id: 0x0001, // VENDOR_QITECH
            product_id: 0x0006, // MACHINE_LASER_V1
        });

        sdr
    };
}
