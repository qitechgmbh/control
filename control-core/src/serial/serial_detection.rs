/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 20.04.2025
*
*@description: This module is responsible for usb detection and validation, specially made with serialport to avoid complexity and size of tokio_serial
*/

use serialport::{SerialPortInfo, SerialPortType, UsbPortInfo};
use std::collections::HashMap;

use super::SerialDeviceIdentification;
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
    fn extract_usb_serial_devices(port_list: Vec<SerialPortInfo>) -> HashMap<String, UsbPortInfo> {
        let mut map = HashMap::new();

        for port in port_list {
            if let SerialPortType::UsbPort(usb_info) = port.port_type {
                map.insert(port.port_name, usb_info);
            }
        }
        map
    }

    /// Scans for connected USB serial devices and updates the internal map.
    /// Returns a clone of the map for quick access.
    pub fn detect_devices() -> HashMap<String, UsbPortInfo> {
        let ports = Self::get_ports();
        let usb_devices = Self::extract_usb_serial_devices(ports);
        usb_devices
    }

    pub fn device_port_exists(device_path: &str) -> bool {
        let ports = Self::get_ports();
        return ports.iter().any(|p| p.port_name == device_path);
    }

    pub fn device_id_exists(sdevid: SerialDeviceIdentification) -> bool {
        let ports = Self::get_ports();
        for port in ports {
            if let SerialPortType::UsbPort(usb_info) = port.port_type {
                if usb_info.vid == sdevid.vendor_id && usb_info.pid == sdevid.product_id {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn get_path_by_id(
        sdevid: SerialDeviceIdentification,
        map: HashMap<String, UsbPortInfo>,
    ) -> Option<String> {
        for (port, usbport) in map.iter() {
            if usbport.vid == sdevid.vendor_id && usbport.pid == sdevid.product_id {
                return Some(port.clone());
            }
        }
        return None;
    }
}
