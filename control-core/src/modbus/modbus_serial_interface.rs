use crate::modbus::{ModbusRequest, ModbusResponse};
use ethercat_hal::io::serial_interface::{SerialEncoding, SerialInterface};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
struct RequestMetaData {
    priority: u32,
    ignored_times: u32,
    /// This is used when a machine for example takes 4ms to process the request
    /// and is added ontop of the standard waiting time for a serial transfer
    extra_delay: Option<u32>,
    no_response_expected: bool,
}

#[derive(Debug)]
pub enum State {
    /// WaitingForResponse is set after sending a request through the serial_interface
    WaitingForResponse,
    /// After Sending a Request we need to wait atleast one ethercat cycle
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
pub struct ModbusSerialInterface {
    serial_interface: SerialInterface,
    baudrate: Option<u32>,
    encoding: Option<SerialEncoding>,
    response: Option<ModbusResponse>,
    state: State,
    last_message_size: usize,
    last_message_delay: u32,
    pub last_message_id: u32,
    no_response_expected: bool,
    request_map: HashMap<u32, ModbusRequest>,
    request_metadata_map: HashMap<u32, RequestMetaData>,
    last_ts: Instant,
}

impl ModbusSerialInterface {
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
            no_response_expected: false,
        }
    }

    pub const fn is_initialized(&self) -> bool {
        !matches!(self.state, State::Uninitialized)
    }

    pub fn add_request(
        &mut self,
        request_id: u32,
        priority: u32,
        request: ModbusRequest,
        no_response_expected: bool,
        delay: Option<u32>,
    ) {
        self.request_map.insert(request_id, request);
        self.request_metadata_map
            .entry(request_id)
            .or_insert_with(|| RequestMetaData {
                priority,
                ignored_times: 0,
                extra_delay: delay,
                no_response_expected,
            });
    }

    /// This is used internally to read the receive buffer of the el6021
    async fn read_modbus_response(&mut self) -> Result<ModbusResponse, anyhow::Error> {
        let raw_response = (self.serial_interface.read_message)()
            .await
            .unwrap_or_default();

        let response = ModbusResponse::try_from(raw_response)?;
        self.last_message_size = response.data.len() + 4;
        self.state = State::WaitingForReceiveAccept;
        Ok(response)
    }

    /// This is used internally to fill the write buffer of the el6021 with the modbus request
    /// Decides what requests to send first by finding the one with the highest priority
    /// For example Highest Priority requests: ResetInverter StopMotor
    async fn send_modbus_request(&mut self) {
        if self.request_map.is_empty() {
            return;
        }

        let (highest_id, delay, no_response_expected) = self
            .request_metadata_map
            .iter_mut()
            .map(|(key, value)| {
                let effective_priority = value.priority + value.ignored_times;
                (
                    *key,
                    effective_priority,
                    value.extra_delay.unwrap_or(0),
                    value.no_response_expected,
                )
            })
            .max_by_key(|(_, priority, _, _)| *priority)
            .map(|(key, _, delay, no_response)| (key, delay, no_response))
            .unwrap(); // Safe because we checked is_empty() above

        let request = &self.request_map[&highest_id];
        let modbus_request: Vec<u8> = request.clone().into();

        if let Err(_) = (self.serial_interface.write_message)(modbus_request.clone()).await {
            tracing::error!("ERROR: serial_interface.write_message has failed");
        } else {
            self.last_message_delay = delay;
            self.last_message_id = highest_id;
            self.no_response_expected = no_response_expected;
        }

        self.state = State::WaitingForRequestAccept;
        self.last_message_size = modbus_request.len();
    }

    pub async fn initialize_communication_settings(&mut self) -> bool {
        let baudrate = (self.serial_interface.get_baudrate)().await;
        let encoding = (self.serial_interface.get_serial_encoding)().await;

        match (baudrate, encoding) {
            (Some(b), Some(e)) => {
                self.baudrate = Some(b);
                self.encoding = Some(e);
                true
            }
            _ => false,
        }
    }

    pub async fn initialize(&mut self) -> bool {
        if matches!(self.state, State::Uninitialized) {
            let success = (self.serial_interface.initialize)().await;
            if success {
                self.state = State::ReadyToSend;
                self.initialize_communication_settings().await;
            }
            success
        } else {
            false
        }
    }

    fn increment_ignored_times(&mut self) {
        for metadata in self.request_metadata_map.values_mut() {
            metadata.ignored_times += 1;
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
        machine_operation_delay: Duration,
        message_size: usize,
    ) -> Option<Duration> {
        let baudrate = self.baudrate?;
        let encoding = self.encoding?;

        let nanoseconds_per_bit = 1_000_000_000 / baudrate as u64;
        let nanoseconds_per_byte = encoding.total_bits() as u64 * nanoseconds_per_bit;

        let transmission_timeout = nanoseconds_per_byte * message_size as u64;
        let silent_time = (nanoseconds_per_byte * 35) / 10; // silent_time is 3.5x of character length

        let total_timeout =
            transmission_timeout + machine_operation_delay.as_nanos() as u64 + silent_time;
        Some(Duration::from_nanos(total_timeout))
    }

    pub const fn get_response(&self) -> Option<&ModbusResponse> {
        self.response.as_ref()
    }

    pub async fn act(&mut self, now_ts: Instant) {
        let elapsed = now_ts.duration_since(self.last_ts);

        let timeout = self.calculate_modbus_rtu_timeout(
            Duration::from_nanos(self.last_message_delay as u64),
            self.last_message_size,
        );

        let Some(timeout) = timeout else { return };

        // Check if we need to wait for timeout (except for specific states)
        if !matches!(
            self.state,
            State::Uninitialized | State::WaitingForReceiveAccept | State::WaitingForRequestAccept
        ) && elapsed < timeout
        {
            return;
        }

        self.last_ts = now_ts;
        self.response = None;

        match self.state {
            State::WaitingForResponse => match self.read_modbus_response().await {
                Ok(response) => {
                    self.response = Some(response);
                }
                Err(_) => {
                    self.response = None;
                    if self.no_response_expected {
                        self.state = State::ReadyToSend;
                    }
                }
            },
            State::ReadyToSend => {
                self.send_modbus_request().await;

                // Remove the sent request
                self.request_map.remove(&self.last_message_id);
                self.request_metadata_map.remove(&self.last_message_id);
                self.increment_ignored_times();
            }
            State::WaitingForReceiveAccept => {
                // Waste at least one Ethercat Cycle here to ensure that request/response stay in sync
                self.state = State::ReadyToSend;
            }
            State::WaitingForRequestAccept => {
                // An empty vec is used to check if we are finished with writing the message
                // This is to keep the Serialinterface more simple
                match (self.serial_interface.write_message)(vec![]).await {
                    Ok(finished) if finished => {
                        self.state = State::WaitingForResponse;
                    }
                    Err(_) => {
                        self.state = State::ReadyToSend;
                    }
                    _ => {} // Still waiting
                }
            }
            State::Uninitialized => {}
        }
    }
}
