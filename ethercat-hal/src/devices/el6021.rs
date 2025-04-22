use super::{NewDevice, SubDeviceIdentityTuple};
use crate::coe::{ConfigurableDevice, Configuration};
use crate::io::serial_interface::SerialInterfaceDevice;
use crate::pdo::{PredefinedPdoAssignment, RxPdo, RxPdoObject, TxPdo, TxPdoObject};
use crate::types::EthercrabSubDevicePreoperational;
use anyhow::anyhow;
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use common::modbus::modbus::SerialEncoding;
use ethercat_hal_derive::{Device, PdoObject};
use ethercat_hal_derive::{RxPdo, TxPdo};

impl std::fmt::Debug for EL6021 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL6021")
    }
}

// Every Preset has 2 bytes at the beginning
// Standard98ByteMdp600 for example is 100bytes big  but has 98 bytes of data
#[derive(Debug, Clone)]
pub enum EL6021PdoPreset {
    /// Legacy 22 Byte MDP 600
    Legacy22ByteMdp600,
    /// Legacy 3 Byte MDP 600
    Legacy3ByteMdp600,
    /// Legacy 5 Byte MDP 600
    Legacy5ByteMdp600,
    /// Standard 22 Byte MDP 600
    Standard22ByteMdp600,
    /// Standard 50 Word MDP 600
    Standard50WordMdp600,
    /// Standard 98 Byte MDP 600
    Standard98ByteMdp600,
    /// No assignment
    None,
}

impl EL6021Baudrate {
    pub fn from(value: u8) -> Option<Self> {
        match value {
            4 => Some(EL6021Baudrate::B2400),
            5 => Some(EL6021Baudrate::B4800),
            6 => Some(EL6021Baudrate::B9600),
            7 => Some(EL6021Baudrate::B19200),
            8 => Some(EL6021Baudrate::B38400),
            9 => Some(EL6021Baudrate::B57600),
            10 => Some(EL6021Baudrate::B115200),
            _ => None,
        }
    }
    pub fn into(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EL6021Baudrate {
    /// 2400 baud (CoE Value: 4)
    B2400 = 4,
    /// 4800 baud (CoE Value: 5)
    B4800 = 5,
    /// 9600 baud (CoE Value: 6) DEFAULT
    B9600 = 6,
    /// 19200 baud (CoE Value: 7)
    B19200 = 7,
    /// 38400 baud (CoE Value: 8)
    B38400 = 8,
    /// 57600 baud (CoE Value: 9)
    B57600 = 9,
    /// 115200 baud (CoE Value: 10)
    B115200 = 10,
}

impl Default for EL6021Configuration {
    fn default() -> Self {
        Self {
            rts_enabled: true,
            xon_off_supported_rx: false,
            xon_on_supported_tx: false,
            enable_transfer_rate_optimization: true,
            fifo_continuous_send_enabled: false,
            half_duplex_enabled: true,
            point_to_point_connection_enabled: false,
            baud_rate: EL6021Baudrate::B19200,
            data_frame: SerialEncoding::Coding8E1,
            rx_buffer_full_notification: 0x0360,
            explicit_baudrate: 19200,
            pdo_assignment: EL6021PdoPreset::Standard22ByteMdp600,
            extended_data_frame: 0x4,
        }
    }
}

impl ConfigurableDevice<EL6021Configuration> for EL6021 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL6021Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        self.rxpdo = config.pdo_assignment.rxpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL6021Configuration {
        self.configuration.clone()
    }
}

/// Configuration structure for the EL6021 module.
#[derive(Debug, Clone)]
pub struct EL6021Configuration {
    /// # 0x8000:01 - Enables Request to Send (RTS) flow control.
    /// - `true` = RTS flow control enabled
    /// - `false` = RTS flow control disabled
    /// default: `true`
    pub rts_enabled: bool,

    /// # 0x8000:02 - Enables XON/XOFF software flow control for received data.
    /// - `true` = XON/XOFF supported for received data
    /// - `false` = XON/XOFF not supported for received data
    /// default: `false`
    pub xon_off_supported_rx: bool,

    /// # 0x8000:03 - Enables XON/XOFF software flow control for transmitted data.
    /// - `true` = XON/XOFF supported for transmitted data
    /// - `false` = XON/XOFF not supported for transmitted data
    /// default: `false`
    pub xon_on_supported_tx: bool,

    /// # 0x8000:04 - Allows continuous transmission of data from the FIFO buffer.
    /// - `true` = FIFO continuous send enabled
    /// - `false` = FIFO continuous send disabled
    /// default: `false`
    pub fifo_continuous_send_enabled: bool,

    pub enable_transfer_rate_optimization: bool,

    /// # 0x8000:06 - Enables half-duplex mode (only applicable to EL6021).
    /// - `true` = Half-duplex mode enabled
    /// - `false` = Half-duplex mode disabled
    /// default: `false`
    pub half_duplex_enabled: bool,

    /// # 0x8000:07 - Enables point-to-point connection mode (only applicable to EL6021).
    /// - `true` = Point-to-point connection mode enabled
    /// - `false` = Point-to-point connection mode disabled
    /// default: `false`
    pub point_to_point_connection_enabled: bool,

    /// # 0x8000:11 - Sets the baud rate (e.g., 9600, 115200).
    /// This value is typically an index referencing predefined baud rates.

    /// default: `0x06`
    pub baud_rate: EL6021Baudrate,

    /// # 0x8000:15 - Defines the data frame format.
    /// This is usually a bitfield representing settings like:
    /// - Bits 0-2: Data bits (5-8)
    /// - Bit 3: Stop bits (1 or 2)
    /// - Bits 4-5: Parity (None, Even, Odd)
    /// default: `0x03` (8N1 format)
    pub data_frame: SerialEncoding,

    /// # 0x8000:1A - Notification threshold for the RX buffer (in bytes).
    /// Determines when the terminal signals the controller that the receive buffer is full.
    /// default: `0x0360`
    pub rx_buffer_full_notification: u16,

    /// # 0x8000:1B - Explicitly sets a custom baud rate (only supported from firmware version FW09).
    /// default: `9600` 0x00000384 0x00002580
    pub explicit_baudrate: u32,

    /// # 8000:1C**
    /// In this object special formats can also be selected in addition
    /// to the usual data frames (e.g. 9N1). Changes to this object
    /// are also adopted in the objects 0x8000:15 and 0x4074.
    pub extended_data_frame: u16,
    pub pdo_assignment: EL6021PdoPreset,
}

// TODO: find a better way than using a pure function to convert
// maybe use a Trait ?
fn convert_serial_encoding(encoding: SerialEncoding) -> u8 {
    match encoding {
        SerialEncoding::Coding7E1 => 1,
        SerialEncoding::Coding7O1 => 2,
        SerialEncoding::Coding7E2 => 9,
        SerialEncoding::Coding7O2 => 10,
        SerialEncoding::Coding8N1 => 3,
        SerialEncoding::Coding8E1 => 4,
        SerialEncoding::Coding8O1 => 5,
        SerialEncoding::Coding8N2 => 11,
        SerialEncoding::Coding8E2 => 12,
        SerialEncoding::Coding8O2 => 13,
        SerialEncoding::Coding8S1 => 18,
        SerialEncoding::Coding8M1 => 19,
    }
}

impl Configuration for EL6021Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        match (self.baud_rate, self.data_frame) {
            (EL6021Baudrate::B2400, SerialEncoding::Coding7E1)
            | (EL6021Baudrate::B4800, SerialEncoding::Coding7O1)
            | (EL6021Baudrate::B9600, SerialEncoding::Coding8N1)
            | (EL6021Baudrate::B19200, SerialEncoding::Coding8E1)
            | (EL6021Baudrate::B38400, SerialEncoding::Coding8O1)
            | (EL6021Baudrate::B57600, SerialEncoding::Coding7E2)
            | (EL6021Baudrate::B115200, SerialEncoding::Coding7O2) => {}
            _ => {
                return Err(anyhow!(
                "ERROR: EL6021Configuration::write_config Baudrate and Coding is not compatible!"
            ))
            }
        }

        //   device.sdo_write(0x8000, 0x1, self.rts_enabled).await?;
        device
            .sdo_write(0x8000, 0x2, self.xon_on_supported_tx)
            .await?;

        device
            .sdo_write(0x8000, 0x3, self.xon_off_supported_rx)
            .await?;

        device
            .sdo_write(0x8000, 0x4, self.fifo_continuous_send_enabled)
            .await?;

        device
            .sdo_write(0x8000, 0x5, self.enable_transfer_rate_optimization)
            .await?;

        device
            .sdo_write(0x8000, 0x6, self.half_duplex_enabled)
            .await?;

        device
            .sdo_write(0x8000, 0x7, self.point_to_point_connection_enabled)
            .await?;
        // baud rate and data frame are only 4 BITs maybe this causes a problem?
        device
            .sdo_write(0x8000, 0x11, self.baud_rate.into())
            .await?;

        device
            .sdo_write(0x8000, 0x15, convert_serial_encoding(self.data_frame))
            .await?;
        device
            .sdo_write(0x8000, 0x1a, self.rx_buffer_full_notification)
            .await?;

        device
            .sdo_write(0x8000, 0x1b, self.explicit_baudrate)
            .await?;

        device
            .sdo_write(0x8000, 0x1c, self.extended_data_frame)
            .await?;
        Ok(())
    }
}

/// The value is accompanied by some metadata.
#[derive(Debug, Clone, PdoObject, PartialEq)]
#[pdo_object(bits = 192)]
pub struct Standard22ByteMdp600Output {
    pub control: u8, // For Standard22ByteMdp600 Output same size as input, but status is ctrl instead
    pub length: u8,
    pub data: [u8; 22],
}

/// The value is accompanied by some metadata.
#[derive(Debug, Clone, PdoObject, PartialEq)]
#[pdo_object(bits = 192)]
pub struct Standard22ByteMdp600Input {
    pub status: u8, // For Standard22ByteMdp600 Output same size as input, but status is ctrl instead
    pub length: u8,
    pub data: [u8; 22],
}

impl Default for Standard22ByteMdp600Input {
    fn default() -> Self {
        Self {
            status: 0,
            length: 0,
            data: [0u8; 22],
        }
    }
}

impl Default for Standard22ByteMdp600Output {
    fn default() -> Self {
        Self {
            control: 0,
            length: 0,
            data: [0u8; 22],
        }
    }
}

impl TxPdoObject for Standard22ByteMdp600Input {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        self.status = bits[0..0 + 8].load_le::<u8>();
        self.length = bits[8..8 + 8].load_le::<u8>();
        let serial_bytes = bits[16..(16 + 22 * 8 as usize)].chunks_exact(8);
        for (i, val) in serial_bytes.enumerate() {
            self.data[i] = val.load_le();
        }
    }
}

impl RxPdoObject for Standard22ByteMdp600Output {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer[0..8].store_le(self.control);
        buffer[8..16].store_le(self.length);
        for (i, &byte) in self.data.iter().take(22).enumerate() {
            buffer[(16 + i * 8)..(16 + (i + 1) * 8)].store_le(byte);
        }
    }
}

// COM TxPDO Map 22Byte
#[derive(Debug, Clone, TxPdo)]
pub struct EL6021TxPdo {
    // 0x1A02
    #[pdo_object_index(0x1A02)]
    pub com_tx_pdo_map_22_byte: Option<Standard22ByteMdp600Input>,
}

// COM RxPDO Map 22Byte
#[derive(Debug, Clone, RxPdo)]
pub struct EL6021RxPdo {
    // 0x1602
    #[pdo_object_index(0x1602)]
    pub com_rx_pdo_map_22_byte: Option<Standard22ByteMdp600Output>,
}

#[derive(Device)]
pub struct EL6021 {
    pub configuration: EL6021Configuration,
    pub txpdo: EL6021TxPdo,
    pub rxpdo: EL6021RxPdo,
    pub output_ts: u64,
    pub input_ts: u64,
}

impl PredefinedPdoAssignment<EL6021TxPdo, EL6021RxPdo> for EL6021PdoPreset {
    fn txpdo_assignment(&self) -> EL6021TxPdo {
        match self {
            EL6021PdoPreset::Standard22ByteMdp600 => EL6021TxPdo {
                com_tx_pdo_map_22_byte: Some(Standard22ByteMdp600Input::default()),
            },
            _ => todo!(),
        }
    }

    fn rxpdo_assignment(&self) -> EL6021RxPdo {
        match self {
            EL6021PdoPreset::Standard22ByteMdp600 => EL6021RxPdo {
                com_rx_pdo_map_22_byte: Some(Standard22ByteMdp600Output::default()),
            },
            _ => todo!(),
        }
    }
}

impl NewDevice for EL6021 {
    fn new() -> Self {
        let configuration: EL6021Configuration = EL6021Configuration::default();
        Self {
            configuration: configuration.clone(),
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            output_ts: 0,
            input_ts: 0,
        }
    }
}

#[derive(Clone)]
pub enum EL6021Port {
    SI1, // Serial
}

pub enum ControlToggle {
    TransmitRequest,
    ReceiveAccepted,
    InitRequest,
}

pub enum StatusToggle {
    TransmitAccepted,
    ReceiveRequest,
    InitAccepted,
}

impl SerialInterfaceDevice<EL6021Port> for EL6021 {
    fn serial_init_request(&mut self, _port: EL6021Port) -> () {
        if let Some(rx_pdo) = &mut self.rxpdo.com_rx_pdo_map_22_byte {
            println!("Initializing EL6021 with new settings...");
            rx_pdo.control |= 0x0004;
            std::thread::sleep(std::time::Duration::from_millis(50));
            if let Some(tx_pdo) = &self.txpdo.com_tx_pdo_map_22_byte {
                if (tx_pdo.status & 0x0004) != 0 {
                    println!("init Accepted {}", tx_pdo.status);
                    rx_pdo.control = 0;
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        }
    }

    fn serial_interface_has_messages(&mut self, _port: EL6021Port) -> bool {
        if let Some(tx_pdo) = &self.txpdo.com_tx_pdo_map_22_byte {
            return (tx_pdo.status & 0x2) != 0;
        }
        return false;
    }

    fn serial_interface_read_message(&mut self, _port: EL6021Port) -> Vec<u8> {
        if !self.serial_interface_has_messages(_port) {
            println!("has no messages");
            return vec![];
        }

        if let Some(tx_pdo) = &mut self.txpdo.com_tx_pdo_map_22_byte {
            let valid_length = tx_pdo.length as usize;
            let received_data = tx_pdo.data[..valid_length.min(22)].to_vec();
            if let Some(rx_pdo) = &mut self.rxpdo.com_rx_pdo_map_22_byte {
                rx_pdo.control ^= 0x2;
            }
            tx_pdo.data.fill(0);
            return received_data;
        } else {
            println!("Error: TxPDO not available");
            vec![]
        }
    }

    fn serial_interface_write_message(&mut self, _port: EL6021Port, message: Vec<u8>) {
        if let Some(rx_pdo) = &mut self.rxpdo.com_rx_pdo_map_22_byte {
            if message.len() > 22 {
                return;
            }
            let mut data_buffer = [0u8; 22];
            let bytes = message.as_slice();
            data_buffer[..message.len()].copy_from_slice(&bytes[..message.len()]);
            rx_pdo.length = message.len() as u8;
            rx_pdo.data = data_buffer;
            if rx_pdo.control == 0 {
                rx_pdo.control ^= 0x1; // Toggle Transmit Request
            }
        }
    }
}

pub const EL6021_VENDOR_ID: u32 = 2;
pub const EL6021_PRODUCT_ID: u32 = 394604626;
pub const EL6021_REVISION_A: u32 = 1376256;
pub const EL6021_IDENTITY_A: SubDeviceIdentityTuple =
    (EL6021_VENDOR_ID, EL6021_PRODUCT_ID, EL6021_REVISION_A);
