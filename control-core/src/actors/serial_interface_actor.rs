use crate::modbus::{ModbusRequest, ModbusResponse};
use ethercat_hal::io::serial_interface::{SerialEncoding, SerialInterface};
use std::{
    collections::HashMap,
    pin::Pin,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
struct RequestMetaData {
    pub priority: u32,
    pub ignored_times: u32,
    /// This is used when a machine for example takes 4ms to process the request
    /// and is added ontop of the standard waiting time for a serial transfer
    pub extra_delay: Option<u32>,
}

#[derive(Debug)]
pub enum State {
    /// WaitingForResponse is set after sending a request through the serial_interface
    WaitingForResponse,
    /// After Sending a Resuest we need to wait atleast one ethercat cycle
    /// After one Cycle we check if el6021 status has transmit accepted toggled
    /// Then we can set state = ReadyToSend
    WaitingForRequestAccept,
    /// After Receiving a Response we need to wait atleast one ethercat cycle
    /// After one Cycle we check if el6021 status has received accepted toggled
    WaitingForReceiveAccept,
    /// ReadyToSend is set after receiving the response from the serial_interface
    ReadyToSend,
    /// Initial State
    Uninitialized,
}

#[derive(Debug)]
pub struct SerialInterfaceActor {
    pub serial_interface: SerialInterface,
    pub baudrate: Option<u32>,
    pub encoding: Option<SerialEncoding>,
    pub response: Option<ModbusResponse>,
    pub state: State,
    pub last_message_size: usize,
    pub last_message_delay: u32,
    pub last_message_id: u32,
    request_map: HashMap<u32, ModbusRequest>,
    request_metadata_map: HashMap<u32, RequestMetaData>,
    last_ts: Instant,
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
            last_message_size: 0,
            last_message_delay: 0,
            state: State::Uninitialized,
            last_ts: Instant::now(),
            last_message_id: 0,
        }
    }

    pub fn is_initialized(&mut self) -> bool {
        if let State::Uninitialized = self.state {
            return false;
        } else {
            return true;
        }
    }

    pub fn add_request(
        &mut self,
        request_id: u32,
        priority: u32,
        request: ModbusRequest,
        delay: Option<u32>,
    ) {
        self.request_map.insert(request_id, request);
        if !self.request_metadata_map.contains_key(&request_id) {
            let meta_data = RequestMetaData {
                priority,
                ignored_times: 0,
                extra_delay: delay,
            };
            self.request_metadata_map.insert(request_id, meta_data);
        }
    }

    /// This is used internally to read the receive buffer of the el6021
    fn read_modbus_response(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<ModbusResponse, anyhow::Error>> + Send + '_>> {
        Box::pin(async move {
            let res: Option<Vec<u8>> = (self.serial_interface.read_message)().await;
            let raw_response = match res {
                Some(res) => res,
                None => {
                    vec![]
                }
            };

            let response: Result<ModbusResponse, _> =
                ModbusResponse::try_from(raw_response.clone());
            match response {
                Ok(result) => {
                    self.last_message_size = result.clone().data.len() + 4;
                    self.state = State::WaitingForReceiveAccept;
                    Ok(result)
                }
                Err(_) => {
                    self.last_message_size = 22;
                    Err(anyhow::anyhow!("error"))
                }
            }
        })
    }

    /// This is used internally to fill the write buffer of the el6021 with the modbus request
    /// Decides what requests to send first by finding the one with the highest priority
    /// For example Highest Priority requests: ResetInverter StopMotor    
    fn send_modbus_request(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            if self.request_map.len() == 0 {
                return;
            };

            let mut highest_prio_request: Option<&ModbusRequest> = None;
            let mut highest_priority: u32 = 0;
            let mut highest_id: &u32 = &0;
            let mut delay: u32 = 0;

            for (key, value) in self.request_metadata_map.iter_mut() {
                // borrowchecker complaining
                let priority = value.priority as u32;
                let ignored_times = value.ignored_times;
                let effective_priority: u32 = priority as u32 + ignored_times;

                if effective_priority > highest_priority {
                    highest_prio_request = self.request_map.get(key);
                    highest_priority = effective_priority;
                    highest_id = key;

                    delay = match value.extra_delay {
                        Some(delay) => delay,
                        None => 0,
                    }
                }
            }

            let request = match highest_prio_request {
                Some(request) => request,
                None => return,
            };
            let modbus_request: Vec<u8> = request.clone().into();

            let res = (self.serial_interface.write_message)(modbus_request.clone()).await;
            match res {
                Ok(_) => {
                    self.last_message_delay = delay;
                    self.last_message_id = *highest_id;
                }
                Err(_) => tracing::error!("ERROR: serial_interface.write_message has failed"),
            }
            self.state = State::WaitingForRequestAccept;
            self.last_message_size = modbus_request.len();
        })
    }

    pub async fn initialize_communication_settings(&mut self) -> bool {
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

        self.baudrate = baudrate;
        self.encoding = encoding;

        return true;
    }

    pub async fn initialize(&mut self) -> bool {
        if let State::Uninitialized = self.state {
            let res = (self.serial_interface.initialize)().await;

            if res == true {
                self.state = State::ReadyToSend;
                self.initialize_communication_settings().await;
                return res;
            }
            return res;
        }
        return false;
    }

    fn set_ignored_times_modbus_requests(&mut self) {
        for (_, value) in self.request_metadata_map.iter_mut() {
            value.ignored_times += 1;
        }
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

    pub fn get_response(&self) -> Option<ModbusResponse> {
        return self.response.clone();
    }

    pub fn act(&mut self, now_ts: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let elapsed: Duration = now_ts.duration_since(self.last_ts);

            let timeout = self.calculate_modbus_rtu_timeout(
                Duration::from_nanos(self.last_message_delay as u64),
                self.last_message_size,
            );

            let timeout = match timeout {
                Some(timeout) => timeout,
                None => return,
            };

            match self.state {
                State::Uninitialized => (),
                _ => {
                    if elapsed < timeout {
                        return;
                    }
                }
            }

            self.last_ts = now_ts;
            self.response = None;

            match self.state {
                State::WaitingForResponse => {
                    let ret = self.read_modbus_response().await;
                    match ret {
                        Ok(ret) => {
                            self.response = Some(ret);
                        }
                        Err(_) => {
                            self.response = None;
                            self.state = State::ReadyToSend;
                            self.last_message_id = 0;
                        }
                    }
                }
                State::ReadyToSend => {
                    self.last_message_id = 0;
                    self.send_modbus_request().await;
                    self.request_map.remove(&self.last_message_id);
                    self.request_metadata_map.remove(&self.last_message_id);
                    self.set_ignored_times_modbus_requests();
                }
                State::WaitingForReceiveAccept => {
                    // Waste atleast one Ethercat Cycle here to ensure that request/response stay in sync
                    self.state = State::ReadyToSend;
                }
                State::WaitingForRequestAccept => {
                    // An empty vec is used to check if we are finished with writing the message
                    // This is to keep the Serialinterface more simple
                    let res = (self.serial_interface.write_message)(vec![]).await;
                    let finished = match res {
                        Ok(res) => res,
                        Err(_) => {
                            self.state = State::ReadyToSend;
                            return;
                        }
                    };

                    if finished == true {
                        self.state = State::WaitingForResponse;
                    }
                }
                _ => (),
            }
        })
    }
}
