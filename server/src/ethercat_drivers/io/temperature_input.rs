use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;

type Value = f32;

pub struct TemperatureInput {
    pub state:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = TemperatureInputState> + Send>> + Send + Sync>,
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

pub trait TemperatureInputDevice<PORTS> {
    fn temperature_input_state(&self, port: PORTS) -> TemperatureInputState;
    fn temparature_input(device: Arc<RwLock<Self>>, port: PORTS) -> TemperatureInput
    where
        Self: Send + Sync + 'static,
        PORTS: Clone + Send + Sync + 'static,
    {
        // build async get closure
        let device2 = device.clone();
        let port2 = port.clone();
        let state = Box::new(move || {
            let device2 = device2.clone();
            let port2 = port2.clone();
            Box::pin(async move {
                let device2_guard = device2.read().await;
                device2_guard.temperature_input_state(port2.clone())
            }) as Pin<Box<dyn Future<Output = TemperatureInputState> + Send + 'static>>
        });

        TemperatureInput {
            // write,
            state,
        }
    }
}
