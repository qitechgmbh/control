use control_core::serial::serial_detection::SerialDetection;
use serialport::UsbPortInfo;
use smol::Task;
use std::{collections::HashMap, time::Duration};

// Async function that runs until at least one serial device is found.
pub async fn find_serial() -> Option<HashMap<String, UsbPortInfo>> {
    smol::Timer::after(Duration::from_secs(1)).await;
    let devices = SerialDetection::detect_devices();
    if !devices.is_empty() {
        return Some(devices);
    } else {
        return None;
    }
}

// Returns a smol::Task that resolves when atleast one device is found.
pub fn start_serial_discovery() -> Task<Option<HashMap<String, UsbPortInfo>>> {
    smol::spawn(find_serial())
}
