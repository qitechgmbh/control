// JUST TESTING STUFF HERE
use std::{pin::Pin, thread, time::Duration};
use ethercat_hal::io::serial_interface::SerialInterface;
use super::Actor;
use chrono::prelude::*;
#[derive(Debug)]
pub struct MitsubishiInverterRS485Actor {
    pub received_response: bool,
    /// Message frames comprise the four message fields shown in the figures above.
    /// A slave recognizes message data as one message when a 3.5 character long no-data time (T1: start/end) is added before
    /// and after the data.
    pub last_bytes_sent : u16, 
    pub bytes_waited : u16,
    pub init_done : bool,
    pub serial_interface: SerialInterface,
    pub last_ts : i64,
}

impl MitsubishiInverterRS485Actor {
    pub fn new(received_response : bool, serial_interface : SerialInterface) -> Self {
        Self { received_response, serial_interface, last_bytes_sent: 0,bytes_waited: 0,init_done:false, last_ts: 0 }
    }
}

fn convert_nanoseconds_to_milliseconds(nanoseconds : u64) -> u64 {
    if nanoseconds == 0 {
        return 0;
    }
    return nanoseconds / 1000000;
}

// For now this assumes 8E1 configuration -> 11 bits per byte
// 8N1 would mean 10 bits per byte 
/*
Monitoring, operation command, frequency
setting (RAM)Less than 12 ms

Parameter read/write, frequency setting
(EEPROM)Less than 30 ms

Parameter clear / All parameter clearLess than 5 s

Reset commandNo reply
*/
enum RequestType {
    OperationCommand,
    ReadWrite,
    ParamClear,
    Reset,
}


fn calculate_modbus_timeout(request_type : RequestType, baudrate : u32, message_size : u32) -> u64 {
    let nanoseconds_per_bit = 1000000 / baudrate;
    let nanoseconds_per_byte = 11 * nanoseconds_per_bit;
    
    let transmission_timeout = nanoseconds_per_byte * message_size;
    let silent_time = ( nanoseconds_per_byte * (35) ) / 10; // silent_time is 3.5x of character length,which is 11 bit for 8E1
    let mut full_timeout:u64 = transmission_timeout as u64;

    match request_type {
        RequestType::OperationCommand => full_timeout += 12 * 1000000, //12ms delay extra 
        RequestType::ReadWrite => full_timeout += 30 * 1000000,//30ms delay extra
        RequestType::ParamClear => full_timeout += 5 * 1000 * 1000000,//5seconds delay
        RequestType::Reset => (),
    }
    
    return full_timeout + silent_time as u64;
}



impl Actor for MitsubishiInverterRS485Actor {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {            
            let now = Utc::now();
         

            if (self.last_ts - now.timestamp_nanos()) > -130000000 {                
                println!("diff last cycle: {}",self.last_ts - (now.timestamp_nanos() ));
                self.last_ts = now.timestamp_nanos();
                return;
            }

            
            if self.init_done == false {
               // (self.serial_interface.initialize_serial)().await;
                self.init_done = true;
            }

            

            //  Example) Read the register values of 41004 (Pr.4) to 41006 (Pr.6) from slave address 17 (H11).
            let mut test_data = vec![0;8]; // Read Holding Register 
            
            //let mut bits : BitSlice<u8, Lsb0>;
            // 8n1 is default meaning 10bits * 3.5 > 35 bits offset at the beginning and end            
            test_data[0] = 0x01; // slave addr
            test_data[1] = 0x03; // function
            
            // 10100000 00101100
            test_data[2] = 0x03; // starting addr Reg H
            test_data[3] = 0xeb; // starting addr Reg L --> 41004 (Pr.4)
           
            test_data[4] = 0x0; // No of Points H                                               test_data[5] = 0x03; // No of Points L
            test_data[5] = 0x1; // No of Points H                                               test_data[5] = 0x03; // No of Points L

            test_data[6] = 244; // Crc Check L
            test_data[7] = 122; // Crc Check H

            
            let mut test_data2 = vec![0;8];

            test_data2[0] = 1;
            test_data2[1] = 3;
            
            test_data2[2] = 3;
            test_data2[3] = 0xeb;
            
            test_data2[4] = 00;
            test_data2[5] = 0x3;


            let X25 : crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_MODBUS);
            let result = X25.checksum(&test_data[..6]);

            let high_byte = (result >> 8) as u8;  // upper 8 bits
            let low_byte = (result & 0xFF) as u8; // lower 8 bits
          //  println!("{} {}", high_byte,low_byte);

            test_data2[6] = low_byte;
            test_data2[7] = high_byte;


            

         //   thread::sleep(Duration::from_millis(130));
            (self.serial_interface.write_message)(test_data2.clone()).await; // keep sending to test
            //self.last_bytes_sent = test_data.clone().len() as u16;
            //self.bytes_waited = 0;
            //thread::sleep(Duration::from_millis(130));
            let res = (self.serial_interface.read_message)().await;
        })
    }
}
