// JUST TESTING STUFF HERE
use std::{pin::Pin};
use ethercat_hal::io::serial_interface::SerialInterface;
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use super::Actor;

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
}

impl MitsubishiInverterRS485Actor {
    pub fn new(received_response : bool, serial_interface : SerialInterface) -> Self {
        Self { received_response, serial_interface, last_bytes_sent: 0,bytes_waited: 0,init_done:false }
    }
}


impl Actor for MitsubishiInverterRS485Actor {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {

            if(self.init_done == false) {
                //self.serial_init_request().await;
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

           // test_data[6] = 0x77; // Crc Check L
           // test_data[7] = 0x2c; // Crc Check L

            (self.serial_interface.write_message)(test_data.clone()).await;
            self.last_bytes_sent = test_data.clone().len() as u16;
            self.bytes_waited = 0;
            let res = (self.serial_interface.read_message)().await;
            


            //let time_out = 9600 / 64
            // 9600 baud 
            // 8 zeichen * 8 --> 64 bits
            // 3.5x 

              
                
                //println!("RESULT: {:?}",res);

                // 9600 baud 
                // 8 zeichen * 8 --> 64 bits
                // 3.5x 
                


            


        })
    }
}
