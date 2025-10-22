/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 20.04.2025
*
*@description: This module is responsible for usb detection and validation, specially made with serialport to avoid complexity and size of tokio_serial
*/

use super::{SerialDeviceIdentification, registry::SerialDeviceRegistry};
use crate::machines::identification::DeviceIdentification;
use serialport::{SerialPortInfo, SerialPortType, UsbPortInfo};
use std::collections::HashMap;
/// Handles scanning and caching of connected USB serial devices.
pub struct SerialDetection {}

impl SerialDetection {
    /// Returns all serial ports on the system (minus macOS "cu" noise).
    fn get_ports() -> Vec<SerialPortInfo> {
        match serialport::available_ports() {
            Ok(ports) => ports
                .into_iter()
                .filter(|port| !port.port_name.contains("cu"))
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Extracts only USB serial devices from the port list.
    fn extract_usb_serial_devices(
        port_list: Vec<SerialPortInfo>,
    ) -> HashMap<SerialDeviceIdentification, UsbPortInfo> {
        let mut map = HashMap::new();

        for port in port_list {
            if let SerialPortType::UsbPort(usb_info) = port.port_type {
                let device_ident = SerialDeviceIdentification {
                    vendor_id: usb_info.vid,
                    product_id: usb_info.pid,
                };
                map.insert(device_ident, usb_info);
            }
        }

        map
    }

    /// Scans for connected USB serial devices and updates the internal map.
    /// Returns a clone of the map for quick access.
    pub fn detect_devices() -> HashMap<SerialDeviceIdentification, UsbPortInfo> {
        let ports = Self::get_ports();
        let usb_devices = Self::extract_usb_serial_devices(ports);
        usb_devices
    }
}
