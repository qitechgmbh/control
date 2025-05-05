/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 20.04.2025
*
*@last_update: 2.05.2025
*@description: This module is responsible for usb detection and validation, specially made with serialport to avoid complexity and size of tokio_serial
*/


use serialport::{SerialPortInfo,SerialPortType,UsbPortInfo};
use std::{
    collections::HashMap,
    sync::Arc,
};
use smol::lock::RwLock;

use control_core::serial::{registry::SerialRegistry, ProductConfig, Serial};
#[derive(Debug)]
struct PortChange {
    added: Vec<(String, UsbPortInfo)>,
    removed: Vec<(String, UsbPortInfo)>,
}
pub struct SerialDetection {
    pub sr: SerialRegistry,
    pub connected_serial_usb: HashMap<String, Result<Arc<RwLock<dyn Serial>>, anyhow::Error>>,
    pub ports: Vec<(String, UsbPortInfo)>,

}


impl SerialDetection {
    pub fn new(sr: SerialRegistry) -> Self {
        SerialDetection {
            sr,
            connected_serial_usb: HashMap::new(),
            ports: Vec::new(),
        }
    }

    /*
    *@return: Option::None if there are no available ports on device
    *@return: Option::Some(Vec) with all available ports
    *
    *@description: detects all available ports and returns them as list
    */
    fn update(&self)-> Option<Vec<SerialPortInfo>>{
        match serialport::available_ports() {
            Ok(ports) => Some(ports),
            Err(e)=> None
        }
    }

    /* @param: port_list -> list of available ports
    *  
    * @return: Vec<(String, UsbPortInfo)> -> list of ports with their names and usb information
    * 
    * @description: This function extracts the USB serial devices from the given list of ports.
    */
    fn extract_usb_serial_devices(&self,port_list:Option<Vec<SerialPortInfo>>)
    -> Vec<(String, UsbPortInfo)>{
        let mut usb_ports: Vec<(String, UsbPortInfo)> = Vec::new();
        match port_list {
            None => {},
            Some(ports_list) =>{
                for port in &ports_list{
                    match &port.port_type {
                        SerialPortType::UsbPort(usb_info) => {
                            usb_ports.push((port.port_name.to_string(), usb_info.clone()));
                        },
                        _ => {},
                    };
                };
            },
        }
        return usb_ports;
    }

    /*@param: old_ports -> list of old ports
    * @param: new_ports -> list of new ports
    * 
    * @return: PortChange -> struct with added and removed ports
    * 
    * @description: This function compares the old and new ports and returns a struct with added and removed ports.
    */
    fn compare_ports(
        &self,
        old_ports: Vec<(String, UsbPortInfo)>,
        new_ports: Vec<(String, UsbPortInfo)>,
        ) -> PortChange {

        let mut added = Vec::new();
        let mut removed = Vec::new();

        for (key, value) in new_ports.iter() {
            if !old_ports.iter().any(|(k, _)| k == key) {
                added.push((key.clone(), value.clone()));
            }
        }
        for (key, value) in old_ports.iter() {
            if !new_ports.iter().any(|(k, _)| k == key) {
                removed.push((key.clone(), value.clone()));
            }
        }

        PortChange { added, removed }
    }
    
    /* @param: connected_serial_usb -> HashMap<String, Result<&'static dyn Serial, anyhow::Error>>
    *      which is used to store the list of available ports with the Connected serial devices
    * @param: delete_list -> list of ports that are needed to be removed
    * 
    * @description: This function removes the given ports from the connected_serial_usb HashMap.
    */
    fn remove_usb_from(
        &mut self,
        delete_list: Vec<(String, UsbPortInfo)>
    ){
        for (path, _) in delete_list {
            self.connected_serial_usb.remove(&path);
        }
    }

    fn retry(
        &mut self,
        check_list: Vec<(String, UsbPortInfo)>){
            for (path, info) in check_list.iter(){
                let conf = ProductConfig{
                    vendor_id: info.vid,
                    product_id: info.pid
                };

                match self.connected_serial_usb.get(path){                    
                    Some(rec) => {
                        match rec {
                            Ok(_) => {},
                            Err(e) => {
                                let ag = self.sr.new_machine(path,&conf);
                                println!("Retrying to connect to port {}: {:?}", path, e);
                                if ag.is_ok(){
                                    self.connected_serial_usb.insert(path.clone(), ag);
                                }
                            }
                        }
                    },
                    None => {}
                }
            }
        }
    
    pub fn cycle(&mut self){
        let up = self.update();
        let usb_ports = self.extract_usb_serial_devices(up);
        let comp = self.compare_ports(self.ports.clone(), usb_ports.clone());
        self.ports = usb_ports;
        for (path,port_info ) in comp.added.iter(){
            let conf = ProductConfig{
                vendor_id: port_info.vid,
                product_id: port_info.pid
            };
            let rec = self.sr.new_machine(path,&conf);
            self.connected_serial_usb.insert(path.clone(), rec);
        }
        self.remove_usb_from(comp.removed);
        let mut check_list = self.ports.clone();
        check_list.retain(|item| !comp.added.contains(item));
        self.retry(check_list);
    }
}

    



    


    
    














