/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 20.04.2025
*
*@description: This module is responsible for serial devices detection and validation, specially made with serialport to avoid complexity and size of tokio_serial
*/

use serialport::{SerialPortInfo,SerialPortType};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};



pub struct SerialDeviceRegistry{
    serial_map: Arc<Mutex<HashMap<u64, SerialPortInfo>>>,
}

impl SerialDeviceRegistry {
    pub fn new() -> Self {
        Self {
            serial_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /*
    *@return: Option::None if there are no available ports on device
    *@return: Returns an empty list if no ports found or error occurred and Vec of SerialPortInfo if ports found
    *
    *@description: detects all available ports and returns them as list
    */
    pub fn update() -> Vec<SerialPortInfo> {
        serialport::available_ports().unwrap_or_else(|_| Vec::new())
    }

    
    /*
    *@param: `ports` - List of available ports.
    *@param: `validator` - Closure that defines whether a port should be selected.
    *
    *@return: Filtered list of ports satisfying the validator.
    *
    *@description: validate list of given ports and returns list of ports that satisfies given parameters
    */
    pub fn validate<F>(ports: Vec<SerialPortInfo>, validator: F) -> Vec<SerialPortInfo>
    where
        F: Fn(&SerialPortInfo) -> bool,
    {
        ports.into_iter()
            .filter(|port| validator(port))
            .collect()
    }


    /*  
    *@param: `validator` - Closure that defines whether a port should be selected.
    *
    *@description: This function updates the list of serial ports in the registry.
     */
    pub fn update_ports<F>(&self, validator: F)
    where
        F: Fn(&SerialPortInfo) -> bool,
    {
        let new_ports = Self::update();
        let valid_ports = Self::validate(new_ports, validator);

        let mut map = self.serial_map.lock().unwrap();

        // Build a new set of current device signatures
        let current_keys: Vec<u64> = valid_ports.iter()
            .map(|port| Self::calculate_port_hash(port)) 
            .collect();

        // Remove entries that are not in the current list
        map.retain(|key, _| current_keys.contains(key));

        // Add new ports if they are not already in the map
        for port in valid_ports {
            let key = Self::calculate_port_hash(&port);

            if !map.contains_key(&key) {
                map.insert(key, port.clone());
            }
        }
    }

    /*
    *@param: `port` - SerialPortInfo to calculate TypeId for.
    *
    *@return: TypeId for the given SerialPortInfo.
    *
    *@description: This function calculates a unique TypeId for the given SerialPortInfo.
    */

    fn calculate_port_hash(port: &SerialPortInfo) -> u64 {
        let mut hasher = DefaultHasher::new();
        port.port_name.hash(&mut hasher);
        if let SerialPortType::UsbPort(ref usb_info) = port.port_type {
            if let Some(serial) = &usb_info.serial_number {
                serial.hash(&mut hasher);
            }
            usb_info.vid.hash(&mut hasher);
            usb_info.pid.hash(&mut hasher);
        }
        hasher.finish()
    }
}




