use control_core::serial::{SerialDeviceIdentification, registry::SerialDeviceRegistry};
use lazy_static::lazy_static;

use crate::serial::devices::dre::Dre;

lazy_static! {
    pub static ref SERIAL_DEVICE_REGISTRY: SerialDeviceRegistry = {
        let mut sdr = SerialDeviceRegistry::new();
        sdr.register::<Dre>(SerialDeviceIdentification {
            vendor_id: 0x0403,
            product_id: 0x6001,
        });
        sdr
    };
}
