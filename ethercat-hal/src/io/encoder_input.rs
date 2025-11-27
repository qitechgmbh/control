use std::{fmt, sync::Arc};

use smol::lock::RwLock;

/// Encoder Input device
///
/// Reads encoder values (counter, frequency, period) from the device.
pub struct EncoderInput {
    /// Read the counter value from the encoder
    get_counter: Box<dyn Fn() -> Result<EncoderInputCounter, anyhow::Error> + Send + Sync>,
    /// Read the frequency value from the encoder (if available)
    get_frequency:
        Box<dyn Fn() -> Result<Option<EncoderInputFrequency>, anyhow::Error> + Send + Sync>,
    /// Read the period value from the encoder (if available)
    get_period: Box<dyn Fn() -> Result<Option<EncoderInputPeriod>, anyhow::Error> + Send + Sync>,
    /// Set the counter value
    set_counter: Box<dyn Fn(u32) -> Result<(), anyhow::Error> + Send + Sync>,
}

impl fmt::Debug for EncoderInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EncoderInput")
    }
}

/// Implement on device that have encoder inputs
impl EncoderInput {
    pub fn new<PORT>(device: Arc<RwLock<dyn EncoderInputDevice<PORT>>>, port: PORT) -> Self
    where
        PORT: Clone + Send + Sync + 'static,
    {
        // build sync get counter closure
        let port_counter = port.clone();
        let device_counter = device.clone();
        let get_counter = Box::new(move || {
            let device = smol::block_on(device_counter.read());
            device.get_counter_value(port_counter.clone())
        });

        // build sync get frequency closure
        let port_frequency = port.clone();
        let device_frequency = device.clone();
        let get_frequency = Box::new(move || {
            let device = smol::block_on(device_frequency.read());
            device.get_frequency(port_frequency.clone())
        });

        // build sync get period closure
        let port_period = port.clone();
        let device_period = device.clone();
        let get_period = Box::new(move || {
            let device = smol::block_on(device_period.read());
            device.get_period(port_period.clone())
        });

        // build sync set counter closure
        let port_set = port;
        let device_set = device;
        let set_counter = Box::new(move |value: u32| {
            let mut device = smol::block_on(device_set.write());
            device.set_counter(port_set.clone(), value)
        });

        Self {
            get_counter,
            get_frequency,
            get_period,
            set_counter,
        }
    }

    /// Get the current counter value of the encoder
    pub fn get_counter_value(&self) -> Result<u32, anyhow::Error> {
        let counter = (self.get_counter)()?;
        Ok(counter.value)
    }

    /// Get the current frequency value of the encoder (if available)
    pub fn get_frequency_value(&self) -> Result<Option<u32>, anyhow::Error> {
        let frequency = (self.get_frequency)()?;
        Ok(frequency.map(|f| f.value))
    }

    /// Get the current period value of the encoder (if available)
    pub fn get_period_value(&self) -> Result<Option<u32>, anyhow::Error> {
        let period = (self.get_period)()?;
        Ok(period.map(|p| p.value))
    }

    /// Set the counter value
    pub fn set_counter_value(&self, value: u32) -> Result<(), anyhow::Error> {
        (self.set_counter)(value)
    }
}

#[derive(Debug, Clone)]
pub struct EncoderInputCounter {
    pub value: u32,
}

#[derive(Debug, Clone)]
pub struct EncoderInputFrequency {
    pub value: u32,
}

#[derive(Debug, Clone)]
pub struct EncoderInputPeriod {
    pub value: u32,
}

pub trait EncoderInputDevice<PORTS>: Send + Sync
where
    PORTS: Clone,
{
    fn get_counter_value(&self, port: PORTS) -> Result<EncoderInputCounter, anyhow::Error>;
    fn get_frequency(&self, port: PORTS) -> Result<Option<EncoderInputFrequency>, anyhow::Error>;
    fn get_period(&self, port: PORTS) -> Result<Option<EncoderInputPeriod>, anyhow::Error>;
    fn set_counter(&mut self, port: PORTS, value: u32) -> Result<(), anyhow::Error>;
}
