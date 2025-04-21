/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 20.04.2025
*
*@description: This module is responsible for usb detection and validation, specially made with serialport to avoid complexity and size of tokio_serial
*/

use serialport::{SerialPortInfo,SerialPortType};

/*
*@return: Option::None if there are no available ports on device
*@return: Option::Some(Vec) with all available ports
*
*@description: detects all available ports and returns them as list
*/
pub async fn update()-> Option<Vec<SerialPortInfo>>{
    match serialport::available_ports() {
        Ok(ports) => Some(ports),
        Err(e)=> None
    }
}
/*
*@param: list -> list with ports
*@param: serial_code -> Serial code of device connected with port that is needed to be detected
*@param: name_pat -> Name pattern or begin of device connected with port that is needed to be detected
*@param: vid -> Vendor ID of device connected with port that is needed to be detected
*@param: pid -> Product ID of device connected with port that is needed to be detected
*
*@return: list of ports that satisfies given parameters
*
*@description: validate list of given ports and returns list of ports that satisfies given parameters
*/
pub async fn validate(list: Option<Vec<SerialPortInfo>>,serial_code: &str, name_pat: &str, vid: u16, pid:u16) -> Vec<SerialPortInfo>{
    let mut result:Vec<SerialPortInfo> = Vec::new();
    match list {
        None => {},
        Some(ports_list) =>{
            for port in &ports_list{
                //port.port_name;
                match &port.port_type {
                    SerialPortType::UsbPort(usb_info) => {
                        // Check serial number if available
                        if let Some(serial) = &usb_info.serial_number {
                            if serial.to_lowercase().starts_with(name_pat) {
                                result.push(port.clone());
                            }
                            if serial == serial_code {
                                result.push(port.clone());
                            }
                        }
                        // Check vendor and product IDs
                        if usb_info.vid == vid && usb_info.pid == pid {
                            result.push(port.clone());
                        }
                    },
                    _ => {},
                };
            };
        },
    }
    return result
}
