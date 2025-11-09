/*use std::{any::Any, sync::Arc};

use smol::lock::RwLock;
use std::fmt::Debug;

use crate::machines::identification::DeviceIdentification;

pub mod serial_detection;

pub trait SerialDevice: Any + Send + Sync + SerialDeviceNew + Debug {}

pub trait SerialDeviceNew {
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Self>>), anyhow::Error>
    where
        Self: Sized;
}

pub trait SerialDeviceThread {
    fn start_thread() -> Result<(), anyhow::Error>;
}

pub struct SerialDeviceNewParams {
    pub path: String,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct SerialDeviceIdentification {
    pub vendor_id: u16,
    pub product_id: u16,
}
*/
