use crate::modbus::{ModbusRequest, ModbusResponse};
use ethercat_hal::io::serial_interface::{SerialEncoding, SerialInterface};
use std::{collections::HashMap, time::Duration};

#[derive(Debug)]
struct RequestMetaData {
    pub priority: u32,
    pub ignored_times: u32,
    /// This is used when a machine for example takes 4ms to process the request
    /// and is added ontop of the standard waiting time for a serial transfer
    pub extra_delay: Option<u32>,
}

#[derive(Debug)]
pub struct SerialInterfaceActor {
    pub serial_interface: SerialInterface,
    pub baudrate: Option<u32>,
    pub encoding: Option<SerialEncoding>,

    request_map: HashMap<u32, ModbusRequest>,
    // the priority
    request_metadata_map: HashMap<u32, RequestMetaData>,
    pub response: Option<ModbusResponse>,
}

pub enum SerialConnectionType {
    Serial,
    ModbusRTU,
    ModbusASCII,
    ModbusTCP,
}

impl SerialInterfaceActor {
    pub fn new(serial_interface: SerialInterface) -> Self {
        Self {
            serial_interface,
            baudrate: None,
            encoding: None,
            request_map: HashMap::new(),
            request_metadata_map: HashMap::new(),
            response: None,
        }
    }

    pub fn add_request(
        &mut self,
        request_id: u32,
        request: ModbusRequest,
        priority: u32,
        delay: Option<u32>,
    ) {
        self.request_map.insert(request_id, request);

        let meta_data = RequestMetaData {
            priority,
            ignored_times: 0,
            extra_delay: delay,
        };

        self.request_metadata_map.insert(request_id, meta_data);
    }

    pub async fn initialize(&self) -> bool {
        let baudrate = (self.serial_interface.get_baudrate)().await;
        let encoding = (self.serial_interface.get_serial_encoding)().await;

        match encoding {
            None => return false,
            Some(_) => (),
        };
        match baudrate {
            None => return false,
            Some(_) => (),
        };
        return true;
    }

    /// Modbus RTU has silent time between frames that needs to be adhered to, if you send before silent_time is over between frames, then there will be lost frames
    /// This silent time is needed to identify the start and end of messages
    /// This function also takes into account the time that the slave we are talking to needs to process our request
    /// bits: amount of bits sent per byte -> for a 8n1 coding: 8 data bits, 0 parity, 1 stop bit (1 start,1 stop) -> 10 bits
    /// machine_operation_delay_nano: Delay for the given operation in nanoseconds as specified by the slaves datasheet (example: mitsubishi csfr84 has 12ms for read write in RAM)
    /// baudrate: bits per second
    /// message_size: size of original message in bytes
    pub fn calculate_modbus_rtu_timeout(
        &self,
        machine_operation_delay_nano: Duration,
        message_size: usize,
    ) -> Option<Duration> {
        let baudrate = match self.baudrate {
            Some(baudrate) => baudrate,
            None => return None,
        };

        let bits = match self.encoding {
            Some(encoding) => encoding.total_bits(),
            None => return None,
        };

        let nanoseconds_per_bit: u64 = (1000000 / baudrate) as u64;
        let nanoseconds_per_byte: u64 = bits as u64 * nanoseconds_per_bit as u64;

        let transmission_timeout: u64 = nanoseconds_per_byte * message_size as u64;
        let silent_time: u64 = (nanoseconds_per_byte * (35)) / 10 as u64; // silent_time is 3.5x of character length,which is 11 bit for 8E1

        let mut full_timeout: u64 = transmission_timeout;
        full_timeout += machine_operation_delay_nano.as_nanos() as u64;
        full_timeout += silent_time;

        return Some(Duration::from_nanos(full_timeout));
    }
}
