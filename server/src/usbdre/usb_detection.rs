/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 20.04.2025
*
*@description: This module is responsible for usb detection and validation, specially made with serialport to avoid complexity and size of tokio_serial
*/

use serialport::{SerialPortInfo,SerialPortType};
use std::collections::HashMap;
use std::collections::HashSet;
/*
*@return: Option::None if there are no available ports on device
*@return: Option::Some(Vec) with all available ports
*
*@description: detects all available ports and returns them as list
*/
pub fn update()-> Option<Vec<SerialPortInfo>>{
    match serialport::available_ports() {
        Ok(ports) => Some(ports),
        Err(e)=> None
    }
}
/*
*@param: save -> HashMap<String, SerialPortInfo> which is used to store the list of available ports
*@param: list -> list with ports
*@param: vid -> Vendor ID of device connected with port that is needed to be detected
*@param: pid -> Product ID of device connected with port that is needed to be detected
*
*@description: validate list of given ports that is usb and satisfies given parameters
*/
pub fn validate_usb(
    save: &mut HashMap<String, SerialPortInfo>,
    list: Option<Vec<SerialPortInfo>>,
    vid: u16, 
    pid:u16)
{
   match list {
       None => { save.clear(); },
       Some(ports_list) =>{
           let mut found_key:Vec<String> = Vec::new();
           for port in &ports_list{
               //port.port_name;
               match &port.port_type {
                   SerialPortType::UsbPort(usb_info) => {
                       // Check vendor and product IDs
                       if usb_info.vid == vid && usb_info.pid == pid {
                           if !save.contains_key(port.port_name.as_str()){
                               save.insert(port.port_name.clone(), port.clone());
                           }
                               found_key.push(port.port_name.clone());
                       }
                   },
                   _ => {},
               };
           };
           let allowed_set: HashSet<_> = found_key.iter().cloned().collect();
           save.retain(|key, _| allowed_set.contains(key))
       },
   }
}
