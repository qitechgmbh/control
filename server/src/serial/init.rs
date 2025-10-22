use crate::socketio::main_namespace::MainNamespaceEvents;
use crate::socketio::main_namespace::machines_event::MachinesEventBuilder;
use crate::{app_state::AppState, machines::registry::MACHINE_REGISTRY};
use anyhow::Result;
use control_core::serial::SerialDeviceIdentification;
use control_core::serial::serial_detection::SerialDetection;
use control_core::socketio::namespace::NamespaceCacheingLogic;
use serialport::UsbPortInfo;
use smol::Timer;
use std::{sync::Arc, thread, time::Duration};

// Returns a completed future, when atleast ONE serial devices were found

pub fn find_serial_future()
-> impl future::Future<Output = HashMap<SerialDeviceIdentification, UsbPortInfo>> {
    future::lazy(|_| async {
        loop {
            let devices = SerialDetection::detect_devices().await;
            if !devices.is_empty() {
                return devices;
            }
            Timer::after(Duration::from_secs(1)).await;
        }
    })
}
