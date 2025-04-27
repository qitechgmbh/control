/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 27.04.2025
*
*@description: This module is responsible for modbus communication with DRE device
*/

use serial::prelude::*;
use std::time::Duration;
use crc::{Crc, CRC_16_MODBUS};

/*
*@param: slave_id -> Slave ID of device
*@param: op -> Operation code
*@param: start_address -> Start address of register
*@param: port -> Serial port to communicate with device
*
*@description: send request to device through modbus rtu protocol for reading some data 
*that depends on given Operation code parameters
*
*@Possible values of Operation code:
* 0x01 - Read Coils
* 0x02 - Read Discrete Inputs
* 0x03 - Read Holding Registers
* 0x04 - Read Input Registers
*/
async pub fn send_read_request(slave_id: u8, op: u8, start_address: u16, port: &mut dyn serial::SerialPort) {
    let mut request: Vec<u8> = vec![
        slave_id,       // Slave Address
        op,       // Function code
        (start_address >> 8) as u8, (start_address & 0xFF) as u8, // Start address
        0x00, 0x01, // Quantity of registers: 1
    ];
    const CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);  // CRC16-ANSI
    let crc = CRC16.checksum(&request);  // Compute CRC checksum
    let crc = crc.to_le_bytes();  // Calculate the CRC
    request.extend_from_slice(&crc); // Append CRC to the request

    port.write_all(&request).unwrap();  // Send the request to the device
    std::thread::sleep(Duration::from_millis(5)); // wait for the device to process the request
}
/*
*@param: port -> Serial port to communicate with device
*
*@return: f32 -> Value read from device
*
*@description: receive response from device through modbus rtu protocol
*/
async pub fn receive(port: &mut dyn serial::SerialPort) -> f32{
    let mut buf: [u8; 256] = [0; 256]; // Buffer to store the response
    loop {
        if let Ok(n) = port.read(&mut buf) {
            if n <= 5  {
                println!("Timeout or no response"); // Check if the response is too short
                continue;
            }
            let reg = u16::from_be_bytes([buf[3], buf[4]]); // Read the register value from the response

            return reg as f32 / 1000.0
        } else {
            println!("Error reading from port");
            continue;
        }
    }
    std::thread::sleep(Duration::from_millis(5)); // wait for the device to process the request
}