/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 27.04.2025
*
*@description: This module is responsible for laser diameter measurement using DRE device
*/
use serial::prelude::*;
use std::time::Duration;
use anyhow::Error;
use control_core::{
    modbus,
    serial::registry::SerialDeviceRegistry,
};

#[derive(Clone)]
struct DreConfig {
    lower_tolerance: f32,
    target_diameter: f32,
    upper_tolerance: f32,
}

struct DreStatus {
    hist_timestamps: Vec<u64>,
    hist_diameter: Vec<f32>,
}
struct Dre {
    port: TTYPort,
    diameter: f32,
    status: DreStatus,
    config: DreConfig,
    path: String,
    failed_request_counter: u8,
}

impl Dre {

    fn new(path: &str, slave_id: u8) -> Result<Dre, Error> {

        ///establishing connection with the serial device
        let mut port = serial::open(path).unwrap();

        port.reconfigure(&|settings| {
            (settings.set_baud_rate(serial::Baud9600).unwrap());
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        }).unwrap();
        port.set_timeout(Duration::from_secs(5)).unwrap();

        let mut failed_request_counter:u8 = 0;
        let mut target_diameter:f32 = 0.0;
        let mut upper_tolerance:f32 = 0.0;
        let mut lower_tolerance:f32 = 0.0;
        let mut diameter:f32 = 0.0;

        //request & responce loop for target diameter
        loop{
            if failed_request_counter >10 {
                return Err(Error::msg("Connection problem with Dre Device" ));
            }
            /// Creating request and sending it to the serial device
            let request = modbus::ModbusRequest{
                slave_id: slave_id,
                function_code: modbus::ModbusFunctionCode::ReadHoldingRegister,
                data: vec![(101 >> 8) as u8, (101 & 0xFF) as u8],
            };
            let request_vec: Vec<u8> = request.into();
            port.write_all(&request_vec).unwrap();

            /// Waiting for response from the serial device
            std::thread::sleep(modbus::calculate_modbus_rtu_timeout(
                8, 
                Duration::from_millis(50), 
                9600,
                8));

            /// Reading response from the serial device
            let value = modbus::receive_data_modbus(&mut port);
            if let Some(value) = value {
                failed_request_counter = 0;
                target_diameter = u16::from_be_bytes([value[0], value[1]]) as f32 / 1000.0;
                break;
            } else {
                failed_request_counter+=1;
                continue;
            }
        }
        
        //request & responce loop for upper tolerance
        loop{
            if failed_request_counter >10 {
                return Err(Error::msg("Connection problem with Dre Device" ));
            }
            /// Creating request and sending it to the serial device
            let request = modbus::ModbusRequest{
                slave_id: slave_id,
                function_code: modbus::ModbusFunctionCode::ReadHoldingRegister,
                data: vec![(102 >> 8) as u8, (102 & 0xFF) as u8],
            };
            let request_vec: Vec<u8> = request.into();
            port.write_all(&request_vec).unwrap();

            /// Waiting for response from the serial device
            std::thread::sleep(modbus::calculate_modbus_rtu_timeout(
                8, 
                Duration::from_millis(50), 
                9600,
                8));

            /// Reading response from the serial device
            let value = modbus::receive_data_modbus(&mut port);
            if let Some(value) = value {
                failed_request_counter=0;
                upper_tolerance = u16::from_be_bytes([value[0], value[1]]) as f32 / 1000.0;
                break;
            }else {
                failed_request_counter+=1;
                continue;
            }
        }

        //request & responce loop for lower tolerance
        loop{
            if failed_request_counter >10 {
                return Err(Error::msg("Connection problem with Dre Device" ));
            }
            /// Creating request and sending it to the serial device
            let request = modbus::ModbusRequest{
                slave_id: slave_id,
                function_code: modbus::ModbusFunctionCode::ReadHoldingRegister,
                data: vec![(103 >> 8) as u8, (103 & 0xFF) as u8],
            };
            let request_vec: Vec<u8> = request.into();
            port.write_all(&request_vec).unwrap();

            /// Waiting for response from the serial device
            std::thread::sleep(modbus::calculate_modbus_rtu_timeout(
                8, 
                Duration::from_millis(50), 
                9600,
                8));

            /// Reading response from the serial device
            let value = modbus::receive_data_modbus(&mut port);
            if let Some(value) = value {
                failed_request_counter=0;
                lower_tolerance = u16::from_be_bytes([value[0], value[1]]) as f32 / 1000.0;
                break;
            } else {
                failed_request_counter+=1;
                continue;
            }
        }

        //request & responce loop for current diameter
        loop{
            if failed_request_counter > 10 {
                return Err(Error::msg("Connection problem with Dre Device" ));
            }
            /// Creating request and sending it to the serial device
            let request = modbus::ModbusRequest{
                slave_id: 1,
                function_code: modbus::ModbusFunctionCode::ReadInputRegister,
                data: vec![(0 >> 8) as u8, (0 & 0xFF) as u8],
            };
            let request_vec: Vec<u8> = request.into();
            port.write_all(&request_vec).unwrap();

            /// Waiting for response from the serial device
            std::thread::sleep(modbus::calculate_modbus_rtu_timeout(
                8, 
                Duration::from_millis(50), 
                9600,
                8));

            /// Reading response from the serial device
            let value = modbus::receive_data_modbus(&mut port);
            if let Some(value) = value {
                failed_request_counter=0;
                diameter = u16::from_be_bytes([value[0], value[1]]) as f32 / 1000.0;
                break;
            } else {
                failed_request_counter+=1;
                continue;
            }
        }
        
        return Ok(Dre{
            port: port,
            diameter: diameter,
            status: DreStatus{
                hist_timestamps: vec![],
                hist_diameter: vec![],
            },
            config: DreConfig{
                lower_tolerance: lower_tolerance,
                target_diameter: target_diameter,
                upper_tolerance: upper_tolerance,
            },
            path: path.to_string(),
            failed_request_counter: failed_request_counter,
        });

    }
    
    fn diameter_request(&self) -> Result<f32, Error> {
        //request & responce loop for current diameter
        loop{
            if self.failed_request_counter > 10 {
                return Err(Error::msg("Connection problem with Dre Device" ));
            }
            /// Creating request and sending it to the serial device
            let request = modbus::ModbusRequest{
                slave_id: 1,
                function_code: modbus::ModbusFunctionCode::ReadInputRegister,
                data: vec![(0 >> 8) as u8, (0 & 0xFF) as u8],
            };
            let request_vec: Vec<u8> = request.into();
            port.write_all(&request_vec).unwrap();

            /// Waiting for response from the serial device
            std::thread::sleep(modbus::calculate_modbus_rtu_timeout(
                8, 
                Duration::from_millis(50), 
                9600,
                8));

            /// Reading response from the serial device
            let value = modbus::receive_data_modbus(&mut port);
            if let Some(value) = value {
                self.failed_request_counter=0;
                diameter = u16::from_be_bytes([value[0], value[1]]) as f32 / 1000.0;
                break;
            } else {
                self.failed_request_counter+=1;
                continue;
            }
        }
    }
}