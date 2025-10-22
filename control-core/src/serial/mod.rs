use std::{any::Any, sync::Arc};

use smol::{channel::Sender, lock::RwLock};
use std::fmt::Debug;

use crate::machines::identification::DeviceIdentification;

pub mod panic;
pub mod registry;
pub mod serial_detection;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct SerialDeviceIdentification {
    pub vendor_id: u16,
    pub product_id: u16,
}
