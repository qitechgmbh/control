use anyhow::Error;
use smol::lock::RwLock;
use std::{fmt, future::Future, pin::Pin, sync::Arc};

pub struct SerialInterface {
    pub has_message: Box<dyn Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>,
    pub write_message: Box<
        dyn Fn(Vec<u8>) -> Pin<Box<dyn Future<Output = Result<bool, Error>> + Send>> + Send + Sync,
    >,
    pub read_message:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = Option<Vec<u8>>> + Send>> + Send + Sync>,

    pub get_baudrate:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = Option<u32>> + Send>> + Send + Sync>,

    pub get_serial_encoding:
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = Option<SerialEncoding>> + Send>> + Send + Sync>,

    pub initialize: Box<dyn Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>,
}

impl fmt::Debug for SerialInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SerialInterface")
    }
}

impl SerialInterface {
    pub fn new<PORT>(device: Arc<RwLock<dyn SerialInterfaceDevice<PORT>>>, port: PORT) -> Self
    where
        PORT: Clone + Send + Sync + 'static,
    {
        let mut port2 = port.clone();
        let mut device2 = device.clone();

        let read_message = Box::new(
            move || -> Pin<Box<dyn Future<Output = Option<Vec<u8>>> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();

                Box::pin(async move {
                    let mut device = device2.write().await;
                    device.serial_interface_read_message(port_clone)
                })
            },
        );

        port2 = port.clone();
        device2 = device.clone();

        let write_message = Box::new(
            move |message: Vec<u8>| -> Pin<Box<dyn Future<Output = Result<bool, Error>> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();
                let message2 = message;

                Box::pin(async move {
                    let mut device = device2.write().await;
                    device.serial_interface_write_message(port_clone, message2)
                })
            },
        );

        port2 = port.clone();
        device2 = device.clone();

        let has_message = Box::new(move || -> Pin<Box<dyn Future<Output = bool> + Send>> {
            let device2 = device2.clone();
            let port_clone = port2.clone();

            Box::pin(async move {
                let mut device = device2.write().await;
                device.serial_interface_has_messages(port_clone)
            })
        });

        port2 = port.clone();
        device2 = device.clone();

        let get_baudrate = Box::new(
            move || -> Pin<Box<dyn Future<Output = Option<u32>> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();

                Box::pin(async move {
                    let device = device2.write().await;
                    device.get_baudrate(port_clone)
                })
            },
        );

        port2 = port.clone();
        device2 = device.clone();

        let get_serial_encoding = Box::new(
            move || -> Pin<Box<dyn Future<Output = Option<SerialEncoding>> + Send>> {
                let device2 = device2.clone();
                let port_clone = port2.clone();

                Box::pin(async move {
                    let device = device2.write().await;
                    device.get_serial_encoding(port_clone)
                })
            },
        );

        port2 = port;
        device2 = device.clone();

        let initialize = Box::new(move || -> Pin<Box<dyn Future<Output = bool> + Send>> {
            let device2 = device2.clone();
            let port_clone = port2.clone();

            Box::pin(async move {
                let mut device = device2.write().await;
                device.serial_interface_initialize(port_clone)
            })
        });

        Self {
            has_message,
            write_message,
            read_message,
            get_baudrate,
            get_serial_encoding,
            initialize,
        }
    }
}

pub trait SerialInterfaceDevice<PORTS>: Send + Sync
where
    PORTS: Clone,
{
    fn serial_interface_read_message(&mut self, port: PORTS) -> Option<Vec<u8>>;
    fn serial_interface_write_message(
        &mut self,
        port: PORTS,
        message: Vec<u8>,
    ) -> Result<bool, Error>;
    fn serial_interface_has_messages(&mut self, port: PORTS) -> bool;
    fn get_serial_encoding(&self, port: PORTS) -> Option<SerialEncoding>;
    fn get_baudrate(&self, port: PORTS) -> Option<u32>;
    fn serial_interface_initialize(&mut self, port: PORTS) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityType {
    Even,
    Odd,
    Space,
    Mark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialEncoding {
    Coding7E1, // 7 data, even parity, 1 stop
    Coding7O1, // 7 data, odd parity, 1 stop
    Coding7E2, // 7 data, even parity, 2 stop
    Coding7O2, // 7 data, odd parity, 2 stop
    Coding8N1, // 8 data, no parity, 1 stop
    Coding8E1, // 8 data, even parity, 1 stop
    Coding8O1, // 8 data, odd parity, 1 stop
    Coding8N2, // 8 data, no parity, 2 stop
    Coding8E2, // 8 data, even parity, 2 stop
    Coding8O2, // 8 data, odd parity, 2 stop
    Coding8S1, // 8 data, space parity, 1 stop
    Coding8M1, // 8 data, mark parity, 1 stop
}

impl SerialEncoding {
    /// Get the number of data bits
    pub const fn data_bits(&self) -> u8 {
        match self {
            Self::Coding7E1 | Self::Coding7O1 | Self::Coding7E2 | Self::Coding7O2 => 7,
            _ => 8,
        }
    }

    /// Get the number of parity bits (0 or 1)
    pub const fn parity_bits(&self) -> u8 {
        match self {
            Self::Coding8N1 | Self::Coding8N2 => 0,
            _ => 1,
        }
    }

    /// Get the parity type
    pub const fn parity_type(&self) -> Option<ParityType> {
        match self {
            Self::Coding7E1 | Self::Coding7E2 | Self::Coding8E1 | Self::Coding8E2 => {
                Some(ParityType::Even)
            }
            Self::Coding7O1 | Self::Coding7O2 | Self::Coding8O1 | Self::Coding8O2 => {
                Some(ParityType::Odd)
            }
            Self::Coding8S1 => Some(ParityType::Space),
            Self::Coding8M1 => Some(ParityType::Mark),
            Self::Coding8N1 | Self::Coding8N2 => None,
        }
    }

    /// Get the number of stop bits
    pub const fn stop_bits(&self) -> u8 {
        match self {
            Self::Coding7E1
            | Self::Coding7O1
            | Self::Coding8N1
            | Self::Coding8E1
            | Self::Coding8O1
            | Self::Coding8S1
            | Self::Coding8M1 => 1,
            Self::Coding7E2
            | Self::Coding7O2
            | Self::Coding8N2
            | Self::Coding8E2
            | Self::Coding8O2 => 2,
        }
    }

    /// Get the total number of bits sent per byte according to the SerialEncoding (including start bit)
    /// For Example: With 8n1 transferring 1 byte over Serial actually transfers 10 bits -> 8 data bits, 0 parity, 1 start bit and 1 stop bit
    pub const fn total_bits(&self) -> u8 {
        // We always have 1 start bit
        1 + self.data_bits() + self.parity_bits() + self.stop_bits()
    }
}
