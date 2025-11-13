use serialport::UsbPortInfo;
use std::collections::HashMap;
/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 20.04.2025
*
*@description: This module is responsible for usb detection and validation, specially made with serialport to avoid complexity and size of tokio_serial
*/
use crate::SerialDeviceIdentification;
use serialport::{SerialPortInfo, SerialPortType};
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

/*

pub async fn start_serial_discovery(app_state: Arc<>) {
    loop {
        let devices = SerialDetection::detect_devices();

        if !devices.is_empty() {
            handle_serial_device_hotplug(app_state.clone(), devices).await;
        }

        smol::Timer::after(Duration::from_secs(1)).await;
    }
}

pub async fn handle_serial_device_hotplug(
    app_state: Arc<SharedState>,
    map: HashMap<String, UsbPortInfo>,
) {
    let laser_ident = SerialDeviceIdentification {
        vendor_id: 0x0403,
        product_id: 0x6001,
    };

    let laser = SerialDetection::get_path_by_id(laser_ident, map);

    let mut unique_ident: Option<MachineIdentificationUnique> = None;

    {
        let machines = app_state.machines.read_arc_blocking();
        for (id, _) in machines.iter() {
            if id.machine_identification == LaserMachine::MACHINE_IDENTIFICATION {
                unique_ident = Some(id.clone());
                break;
            }
        }
    }

    // Machine isnt connected, so add it back
    if laser.is_some() && unique_ident.is_none() {
        let serial_params = SerialDeviceNewParams {
            path: laser.unwrap(),
        };
        let _ = match Laser::new_serial(&serial_params) {
            Ok((device_identification, serial_device)) => {
                {
                    let mut machines = app_state.machines.write().await;
                    machines.add_serial_device(
                        &device_identification,
                        serial_device,
                        &MACHINE_REGISTRY,
                        app_state.socketio_setup.socket_queue_tx.clone(),
                        Arc::downgrade(&app_state.machines),
                    );
                }

                let app_state_event = app_state.clone();
                let main_namespace = &mut app_state_event
                    .socketio_setup
                    .namespaces
                    .write()
                    .await
                    .main_namespace;
                let event = MachinesEventBuilder().build(app_state_event.clone());
                main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
            }
            _ => (),
        };
    } else if laser.is_none() && unique_ident.is_some() {
        let serial_params = SerialDeviceNewParams {
            path: "".to_string(),
        };

        let _ = match Laser::new_serial(&serial_params) {
            Ok((device_identification, _)) => {
                app_state
                    .machines
                    .write()
                    .await
                    .remove_serial_device(&device_identification);
                drop(app_state.machines.clone());

                app_state
                    .machines
                    .write_arc()
                    .await
                    .serial_machines
                    .remove(&unique_ident.clone().unwrap());

                app_state
                    .machines
                    .write_arc()
                    .await
                    .ethercat_machines
                    .remove(&unique_ident.clone().unwrap());

                drop(app_state.machines.clone());

                let main_namespace = &mut app_state
                    .socketio_setup
                    .namespaces
                    .write()
                    .await
                    .main_namespace;

                let event = MachinesEventBuilder().build(app_state.clone());
                main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
            }
            _ => (),
        };
    }
}


*/
