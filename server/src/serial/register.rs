use lazy_static::lazy_static;
use control_core::serial::registry::SerialRegistry;
use crate::serial::dre_config::{DRE_PID, DRE_VID};
use crate::serial::dre::Dre;
use control_core::serial::ProductConfig;

lazy_static! {
    pub static ref SERIAL_REGISTRY: SerialRegistry = {
        let mut sdi = SerialRegistry::new();
        sdi.register::<Dre>(ProductConfig {vendor_id: DRE_VID, product_id:DRE_PID});
        sdi
    };
}
