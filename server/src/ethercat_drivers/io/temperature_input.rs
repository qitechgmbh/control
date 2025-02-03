use parking_lot::RwLock;
use std::sync::Arc;

type Value = f32;

pub struct TemperatureInput {
    pub state: Box<dyn Fn() -> TemperatureInputState + Send + Sync>,
}

impl TemperatureInput {
    pub fn new<PORTS>(
        device: Arc<RwLock<dyn TemperatureInputDevice<PORTS>>>,
        port: PORTS,
    ) -> TemperatureInput
    where
        PORTS: Clone + Send + Sync + 'static,
    {
        // build async get closure
        let device2 = device.clone();
        let port2 = port.clone();
        let state = Box::new(move || {
            let device2_guard = device2.read();
            device2_guard.temperature_input_state(port2.clone())
        });

        TemperatureInput { state }
    }
}

#[derive(Debug, Clone)]
pub struct TemperatureInputState {
    /// Nanosecond timestamp
    pub input_ts: u64,
    /// Temperature in degrees Celsius (°C) with a resolution of 0.1°C
    pub value: Value,
    /// Under-voltage error
    pub status_undervoltage: bool,
    /// Over-voltage error
    pub status_overvoltage: bool,
    /// Configured limit 1
    pub limit_1: TemperatureInputLimit,
    /// Configured limit 2
    pub limit_2: TemperatureInputLimit,
    /// Error flag
    pub error: bool,
    /// if the TxPdu sstate is valid
    pub valid: TemperatureInputValid,
    /// if the TxPdu is toggled
    pub toggle: bool,
}

#[derive(Debug, Clone)]
pub enum TemperatureInputLimit {
    NotActive,
    Greater,
    Smaller,
    Equal,
}

impl TemperatureInputLimit {
    pub fn new(value: u8) -> Self {
        match value {
            0b00 => TemperatureInputLimit::NotActive,
            0b01 => TemperatureInputLimit::Greater,
            0b10 => TemperatureInputLimit::Smaller,
            0b11 => TemperatureInputLimit::Equal,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TemperatureInputValid {
    Valid,
    Invalid,
}

impl TemperatureInputValid {
    pub fn new(value: u8) -> Self {
        match value {
            0 => TemperatureInputValid::Valid,
            1 => TemperatureInputValid::Invalid,
            _ => unreachable!(),
        }
    }
}

pub trait TemperatureInputDevice<PORTS>: Send + Sync {
    fn temperature_input_state(&self, port: PORTS) -> TemperatureInputState;
}
