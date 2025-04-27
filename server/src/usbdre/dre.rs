/*
*@author: Alisher Darmenov
*@company: QiTech
*@created: 27.04.2025
*
*@description: This module is responsible for laser diameter measurement using DRE device
*/
use serial::prelude::*;
use std::time::Duration;
use crc::{Crc, CRC_16_MODBUS};

pub mod config;
pub mod usb_detection;
pub mod modbus;

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
    diameter: f32,
    status: DreStatus,
    config: DreConfig,
    path: String,
    failed_request_counter: u8,
}

impl Dre {
    /* @parameter: path, the directory to DRE MODBus 
    *  @description: the 
    *
    */


    async fn new(path: &str){
        let mut port = serial::open(port).unwrap();
        port.reconfigure(&|settings| {
            (settings.set_baud_rate(serial::Baud38400).unwrap());
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        })
        .unwrap();

        port.set_timeout(Duration::from_secs(3600)).unwrap();



    }
}