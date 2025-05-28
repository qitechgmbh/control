use control_core::serial::{SerialDeviceIdentification, registry::SerialDeviceRegistry};
use lazy_static::lazy_static;

use crate::serial::devices::dre::Dre;

#[cfg(feature = "mock-machine")]
use crate::serial::devices::mock::MockSerialDevice;

lazy_static! {
    pub static ref SERIAL_DEVICE_REGISTRY: SerialDeviceRegistry = {
        let mut sdr = SerialDeviceRegistry::new();
        sdr.register::<Dre>(SerialDeviceIdentification {
            vendor_id: 0x0403,
            product_id: 0x6001,
        });
        
        // Register MockSerialDevice when mock-machine feature is enabled
        #[cfg(feature = "mock-machine")]
        sdr.register::<MockSerialDevice>(SerialDeviceIdentification {
            vendor_id: 0x0001, // VENDOR_QITECH
            product_id: 0x0007, // MACHINE_MOCK
        });
        
        sdr
    };
}
