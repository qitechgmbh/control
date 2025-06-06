/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 20.04.2025
*
*@description: This module is responsible for usb detection and validation, specially made with serialport to avoid complexity and size of tokio_serial
*/

use serialport::{SerialPortInfo, SerialPortType, UsbPortInfo};
use smol::{
    channel::{Receiver, Sender, unbounded},
    lock::RwLock,
};
use std::{collections::HashMap, sync::Arc};

use crate::{
    helpers::compare_lists::compare_lists, machines::identification::DeviceIdentification,
};

use super::{
    SerialDevice, SerialDeviceIdentification, SerialDeviceNewParams, registry::SerialDeviceRegistry,
};

pub struct SerialDetection<'serialdeviceregistry> {
    pub serial_device_registry: &'serialdeviceregistry SerialDeviceRegistry,
    pub ports: HashMap<
        String,
        (
            UsbPortInfo,
            DeviceIdentification,
            Arc<RwLock<dyn SerialDevice>>,
        ),
    >,
    pub device_removal_signal_rx: Receiver<(String, anyhow::Error)>,
    pub device_removal_signal_tx: Sender<(String, anyhow::Error)>,
}

impl<'serialdeviceregistry> SerialDetection<'serialdeviceregistry> {
    pub fn new(serial_device_registry: &'serialdeviceregistry SerialDeviceRegistry) -> Self {
        let (device_removal_signal_tx, device_removal_signal_rx) =
            unbounded::<(String, anyhow::Error)>();
        SerialDetection {
            serial_device_registry,
            ports: HashMap::new(),
            device_removal_signal_rx,
            device_removal_signal_tx,
        }
    }

    /*
     *@return: Option::None if there are no available ports on device
     *@return: Option::Some(Vec) with all available ports
     *
     *@description: detects all available ports and returns them as list
     */
    fn get_ports() -> Vec<SerialPortInfo> {
        let all = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(_) => return Vec::new(),
        };

        // exclude ports that contai "cu" in their name
        all.into_iter()
            .filter(|port| !port.port_name.contains("cu"))
            .collect::<Vec<_>>()
    }

    /* @param: port_list -> list of available ports
     *
     * @return: Vec<(String, UsbPortInfo)> -> list of ports with their names and usb information
     *
     * @description: This function extracts the USB serial devices from the given list of ports.
     */
    fn extract_usb_serial_devices(
        &self,
        port_list: Vec<SerialPortInfo>,
    ) -> Vec<(String, UsbPortInfo)> {
        let mut usb_ports: Vec<(String, UsbPortInfo)> = Vec::new();

        for port in port_list {
            match &port.port_type {
                SerialPortType::UsbPort(usb_info) => {
                    usb_ports.push((port.port_name.to_string(), usb_info.clone()));
                }
                _ => {}
            };
        }

        return usb_ports;
    }

    pub async fn check_ports(&mut self) -> CheckPortsResult {
        // get available ports
        let ports = Self::get_ports();

        // extract all ports that are usb ports
        let usb_ports = self.extract_usb_serial_devices(ports);

        let last_usb_ports = self
            .ports
            .iter()
            .map(|last_usb_port| (last_usb_port.0.clone(), last_usb_port.1.0.clone()))
            .collect::<Vec<_>>();

        let usb_ports_diff = compare_lists(&last_usb_ports, &usb_ports);

        let mut result = CheckPortsResult {
            added: Vec::new(),
            removed: Vec::new(),
        };

        // reamove ports
        for removed in usb_ports_diff.removed {
            if let Some((_, device_identification, _)) = self.ports.remove(&removed.0) {
                result.removed.push(device_identification);
            }

            tracing::trace!("Removed port: {}", removed.0);
        }

        // add new ports
        for added in usb_ports_diff.added {
            // add the port to the list
            let serial_device_identification = SerialDeviceIdentification {
                vendor_id: added.1.vid,
                product_id: added.1.pid,
            };

            let device_result = self.serial_device_registry.new_serial_device(
                &SerialDeviceNewParams {
                    path: added.0.clone(),
                    device_thread_panic_tx: self.device_removal_signal_tx.clone(),
                },
                &serial_device_identification,
            );

            // only if created device driver sucessfully
            if let Ok((device_identification, device)) = device_result {
                // add the device to the ports list
                self.ports.insert(
                    added.0.clone(),
                    (
                        added.1.clone(),
                        device_identification.clone(),
                        device.clone(),
                    ),
                );

                // add to result list
                result.added.push((device_identification, device.clone()));

                tracing::trace!("Added port: {}", added.0);
            }
        }

        result
    }

    pub async fn check_remove_signals(&mut self) -> Vec<DeviceIdentification> {
        let mut removed_signals: Vec<DeviceIdentification> = Vec::new();
        match self.device_removal_signal_rx.try_recv() {
            Ok((path, error)) => {
                // remove the device when the tuple positon 1 equals signal
                if let Some(info) = self.ports.get(&path.clone()) {
                    removed_signals.push(info.1.clone());
                };
                self.ports.remove(&path);

                tracing::trace!("Removed device: {:?}: {}", path, error);
            }
            Err(_) => {}
        }
        removed_signals
    }
}

#[derive(Debug)]
pub struct CheckPortsResult {
    pub added: Vec<(DeviceIdentification, Arc<RwLock<dyn SerialDevice>>)>,
    pub removed: Vec<DeviceIdentification>,
}
