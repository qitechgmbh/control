use std::sync::Arc;

use lazy_static::lazy_static;
use control_core::serial::registry::SerialRegistry;
use smol::lock::RwLock;
use crate::serial::dre_config::{DRE_PID, DRE_VID};
use crate::machines::dre::Dre;
use control_core::serial::ProductConfig;
use crate::serial::serial_detection::SerialDetection;
lazy_static! {
    pub static ref SERIAL_DETECTION: Arc<RwLock<SerialDetection>> = {
        let mut sdi = SerialRegistry::new();
        sdi.register::<Dre>(ProductConfig {vendor_id: DRE_VID, product_id:DRE_PID});
        Arc::new(RwLock::new(SerialDetection::new(sdi)))
    };
}
