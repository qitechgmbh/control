use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use crate::coe::{ConfigurableDevice, Configuration};
use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;
use crate::io::serial_interface::{SerialEncoding, SerialInterfaceDevice};
use crate::pdo::{PredefinedPdoAssignment, RxPdo, RxPdoObject, TxPdo, TxPdoObject};
use anyhow::{Error, anyhow};
use bitvec::field::BitField;
use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;

use ethercat_hal_derive::{EthercatDevice, PdoObject};
use ethercat_hal_derive::{RxPdo, TxPdo};

impl std::fmt::Debug for EL6021 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL6021")
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

// Every Preset has 2 bytes at the beginning
// Standard98ByteMdp600 for example is 100bytes big  but has 98 bytes of data
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

// Implement From<u8> for EL6021Baudrate
impl TryFrom<u8> for EL6021Baudrate {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            4 => Ok(Self::B2400),
            5 => Ok(Self::B4800),
            6 => Ok(Self::B9600),
            7 => Ok(Self::B19200),
            8 => Ok(Self::B38400),
            9 => Ok(Self::B57600),
            10 => Ok(Self::B115200),
            _ => Err(anyhow::anyhow!(
                "Error: specified Baudrate is not supported!"
            )),
        }
    }
}

impl From<EL6021Baudrate> for u8 {
    fn from(baudrate: EL6021Baudrate) -> Self {
        baudrate as Self
    }
}

impl From<EL6021Baudrate> for u32 {
    fn from(value: EL6021Baudrate) -> Self {
        match value {
            EL6021Baudrate::B2400 => 2400,
            EL6021Baudrate::B4800 => 4800,
            EL6021Baudrate::B9600 => 9600,
            EL6021Baudrate::B19200 => 19200,
            EL6021Baudrate::B38400 => 38400,
            EL6021Baudrate::B57600 => 57600,
            EL6021Baudrate::B115200 => 115200,
        }
    }
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
            pdo_assignment: EL6021PdoPreset::Standard22ByteMdp600,
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// 0x8000:05 - Enable Transfer Rate optimization
    /// Transfer rate optimization switched on:
    /// The content of the input buffer is automatically
    /// transferred into the process image if
    /// • no further byte was received for approx. 16
    /// bit times (i.e. the time it would have taken to
    /// receive 2 bytes) after data were received;
    /// • the process image is filled
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
    /// This is usually a bitfield representing settings like this:
    /// - Bits 0-2: Data bits (5-8)
    /// - Bit 3: Stop bits (1 or 2)
    /// - Bits 4-5: Parity (None, Even, Odd)
    /// default: `0x03` (8N1 format)
    pub data_frame: SerialEncoding,

    /// # 0x8000:1A - Notification threshold for the RX buffer (in bytes).
    /// Determines when the terminal signals the controller that the receive buffer is full.
    /// default: `0x0360`
    pub rx_buffer_full_notification: u16,
    pub pdo_assignment: EL6021PdoPreset,
}

const fn convert_serial_encoding(encoding: SerialEncoding) -> u8 {
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
                    "ERROR: EL6021Configuration::write_config Baudrate and Encoding is not compatible!"
                ));
            }
        }

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

        let baudrate_coe_value = u8::from(self.baud_rate);
        device.sdo_write(0x8000, 0x11, baudrate_coe_value).await?;

        device
            .sdo_write(0x8000, 0x15, convert_serial_encoding(self.data_frame))
            .await?;
        device
            .sdo_write(0x8000, 0x1a, self.rx_buffer_full_notification)
            .await?;

        self.pdo_assignment
            .txpdo_assignment()
            .write_config(device)
            .await?;
        self.pdo_assignment
            .rxpdo_assignment()
            .write_config(device)
            .await?;

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 192)]
pub struct Standard22ByteMdp600Output {
    pub control: EL6021Control,
    pub length: u8,
    pub data: [u8; 22],
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct EL6021Status {
    pub transmit_accepted: bool,
    pub receive_request: bool,
    pub init_accepted: bool,
    pub buffer_full: bool,
    pub parity_error: bool,
    pub framing_error: bool,
    pub overrun_error: bool,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct EL6021Control {
    pub transmit_request: bool,
    pub received_acepted: bool,
    pub init_request: bool,
}

/// The value is accompanied by some metadata.
#[derive(Default, Debug, Clone, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 192)]
pub struct Standard22ByteMdp600Input {
    pub status: EL6021Status,
    pub length: u8,
    pub data: [u8; 22],
}

impl TxPdoObject for Standard22ByteMdp600Input {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        self.status.transmit_accepted = bits[0];
        self.status.receive_request = bits[1];
        self.status.init_accepted = bits[2];
        self.status.buffer_full = bits[3];
        self.status.parity_error = bits[4];
        self.status.framing_error = bits[5];
        self.status.overrun_error = bits[6];
        // Bit7 is reserved/unused
        self.length = bits[8..8 + 8].load_le::<u8>();
        let serial_bytes = bits[16..(16 + 22 * 8_usize)].chunks_exact(8);
        for (i, val) in serial_bytes.enumerate() {
            self.data[i] = val.load_le();
        }
    }
}

impl RxPdoObject for Standard22ByteMdp600Output {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer.set(0, self.control.transmit_request);
        buffer.set(1, self.control.received_acepted);
        buffer.set(2, self.control.init_request);
        //bit 3-7 is unused
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

#[derive(EthercatDevice)]
pub struct EL6021 {
    pub configuration: EL6021Configuration,
    pub txpdo: EL6021TxPdo,
    pub rxpdo: EL6021RxPdo,
    is_used: bool,
    pub output_ts: u64,
    pub input_ts: u64,
    pub initialized: bool,
    pub has_messages_last_toggle: bool,
}

impl EthercatDeviceProcessing for EL6021 {}

impl PredefinedPdoAssignment<EL6021TxPdo, EL6021RxPdo> for EL6021PdoPreset {
    fn txpdo_assignment(&self) -> EL6021TxPdo {
        match self {
            Self::Standard22ByteMdp600 => EL6021TxPdo {
                com_tx_pdo_map_22_byte: Some(Standard22ByteMdp600Input::default()),
            },
            _ => todo!(),
        }
    }

    fn rxpdo_assignment(&self) -> EL6021RxPdo {
        match self {
            Self::Standard22ByteMdp600 => EL6021RxPdo {
                com_rx_pdo_map_22_byte: Some(Standard22ByteMdp600Output::default()),
            },
            _ => todo!(),
        }
    }
}

impl NewEthercatDevice for EL6021 {
    fn new() -> Self {
        let configuration: EL6021Configuration = EL6021Configuration::default();
        Self {
            configuration: configuration.clone(),
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            is_used: false,
            output_ts: 0,
            input_ts: 0,
            has_messages_last_toggle: false,
            initialized: false,
        }
    }
}

#[derive(Clone)]
pub enum EL6021Port {
    SI1, // Serial
}

impl SerialInterfaceDevice<EL6021Port> for EL6021 {
    fn serial_interface_has_messages(&mut self, _port: EL6021Port) -> bool {
        if let Some(tx_pdo) = &mut self.txpdo.com_tx_pdo_map_22_byte {
            // Only check if the bit has changed, don't update our state yet
            return tx_pdo.status.receive_request != self.has_messages_last_toggle;
        }
        false
    }

    fn serial_interface_read_message(&mut self, _port: EL6021Port) -> Option<Vec<u8>> {
        if !self.serial_interface_has_messages(_port) {
            return None;
        }

        let tx_pdo_opt = &mut self.txpdo.com_tx_pdo_map_22_byte;
        let tx_pdo = match tx_pdo_opt {
            Some(tx_pdo_opt) => tx_pdo_opt,
            None => return None,
        };

        let rx_pdo_opt = &mut self.rxpdo.com_rx_pdo_map_22_byte;
        let rx_pdo = match rx_pdo_opt {
            Some(rx_pdo_opt) => rx_pdo_opt,
            None => return None,
        };

        let valid_length = tx_pdo.length as usize;
        let received_data = tx_pdo.data[..valid_length.min(22)].to_vec();

        if received_data.is_empty() {
            return None;
        }

        // Update our stored state of the toggle bit AFTER reading the data
        self.has_messages_last_toggle = tx_pdo.status.receive_request;
        rx_pdo.control.received_acepted = !rx_pdo.control.received_acepted;

        Some(received_data)
    }

    fn serial_interface_write_message(
        &mut self,
        _port: EL6021Port,
        message: Vec<u8>,
    ) -> Result<bool, Error> {
        let tx_pdo_opt = &mut self.txpdo.com_tx_pdo_map_22_byte;
        let tx_pdo = match tx_pdo_opt {
            Some(tx_pdo_opt) => tx_pdo_opt,
            None => return Err(anyhow::anyhow!("TXPDO Unavailable!!")),
        };

        let rx_pdo_opt = &mut self.rxpdo.com_rx_pdo_map_22_byte;
        let rx_pdo = match rx_pdo_opt {
            Some(rx_pdo_opt) => rx_pdo_opt,
            None => return Err(anyhow::anyhow!("RXPDO Unavailable!!")),
        };

        // perhaps the 22 could be a constant
        if message.len() > 22 {
            return Err(anyhow::anyhow!(
                "Message is too long for RxPdo Buffer of 22 bytes!"
            ));
        }
        // If we write a message of len zero, then this means we are waiting for our write_message to finish on the EL6021
        if message.is_empty() {
            return Ok(rx_pdo.control.transmit_request == tx_pdo.status.transmit_accepted);
        }

        let mut data_buffer = [0u8; 22];
        let bytes = message.as_slice();
        data_buffer[..message.len()].copy_from_slice(&bytes[..message.len()]);
        rx_pdo.length = message.len() as u8;
        rx_pdo.data = data_buffer;
        rx_pdo.control.transmit_request = !rx_pdo.control.transmit_request;
        Ok(true)
    }

    fn get_baudrate(&self, _port: EL6021Port) -> Option<u32> {
        let baudrate: u32 = self.configuration.baud_rate.into();
        Some(baudrate)
    }

    fn get_serial_encoding(&self, _port: EL6021Port) -> Option<SerialEncoding> {
        Some(self.configuration.data_frame)
    }

    /// For el6021 this returns false for as long as the Initialization takes
    /// When its finished it returns true    
    /// Every step of the init has to be done in an EtherCatCycle
    fn serial_interface_initialize(&mut self, port: EL6021Port) -> bool {
        match port {
            EL6021Port::SI1 => {
                let rxpdo_opt = &mut self.rxpdo.com_rx_pdo_map_22_byte;

                let rxpdo = match rxpdo_opt {
                    Some(rxpdo_opt) => rxpdo_opt,
                    None => return false,
                };
                let txpdo_opt = &mut self.txpdo.com_tx_pdo_map_22_byte;
                let txpdo = match txpdo_opt {
                    Some(txpdo_opt) => txpdo_opt,
                    None => return false,
                };

                /*
                Initialization was accepted
                init_accepted 1: Initialization was completed by the terminal.
                init_request 1: The controller requests terminal for initialization. The
                    transmit and receive functions will be blocked, the FIFO
                    pointer will be reset and the interface will be initialized with
                    the values of the responsible Settings object. The execution
                    of the initialization will be acknowledged by the terminal
                    with the ‘Init accepted’ bit.
                */
                if rxpdo.control.init_request && txpdo.status.init_accepted {
                    rxpdo.control.init_request = false;
                    return false;
                }

                /*
                    This is the initial state
                    init_accepted 0: Initialization was completed by the terminal.
                    init_request 0: The terminal is ready again for serial data exchange.
                */
                if !rxpdo.control.init_request && !txpdo.status.init_accepted && !self.initialized {
                    rxpdo.control.init_request = true;
                    self.initialized = true;
                    return false;
                }

                /*
                    init_accepted 1: Initialization was completed by the terminal.
                    init_request 0: The terminal is ready again for serial data exchange.
                */
                if !rxpdo.control.init_request && txpdo.status.init_accepted {
                    return false;
                }

                /*
                    If both init_request and init_accepted == false, initialization is complete
                    init_accepted 0: The controller once again requests the terminal to prepare for serial data exchange.
                    init_request 0: The terminal is ready again for serial data exchange.
                */
                if !rxpdo.control.init_request && !txpdo.status.init_accepted && self.initialized {
                    // set inital state of the toggle
                    self.has_messages_last_toggle = txpdo.status.receive_request;
                    return true;
                }

                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;

    #[test]
    fn test_standard_22_byte_mdp600_input_read() {
        let mut bits: BitVec<u8, Lsb0> = BitVec::with_capacity(192);
        bits.resize(192, false);
        bits[0..8].store_le(0b101010101); // status
        bits[8..16].store_le(0x16u8); // length (22)
        for i in 0..22 {
            bits[(16 + i * 8)..(16 + (i + 1) * 8)].store_le((i + 1) as u8); // data
        }

        let mut input = Standard22ByteMdp600Input::default();

        input.read(bits.as_bitslice());
        assert_eq!(input.status.transmit_accepted, true);
        assert_eq!(input.status.receive_request, false);
        assert_eq!(input.status.init_accepted, true);
        assert_eq!(input.status.buffer_full, false);
        assert_eq!(input.status.parity_error, true);
        assert_eq!(input.status.receive_request, false);
        assert_eq!(input.length, 0x16);
        for i in 0..22 {
            assert_eq!(input.data[i], (i + 1) as u8);
        }
    }

    #[test]
    fn test_standard_22_byte_mdp600_output_write() {
        let control = EL6021Control {
            transmit_request: true,
            received_acepted: false,
            init_request: true,
        };

        let output = Standard22ByteMdp600Output {
            control: control,
            length: 0x16,
            data: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
            ],
        };
        let mut buffer: BitVec<u8, Lsb0> = BitVec::with_capacity(192);
        buffer.resize(192, false);

        output.write(buffer.as_mut_bitslice());

        assert_eq!(buffer[0..8].load_le::<u8>(), 0b101);
        assert_eq!(buffer[8..16].load_le::<u8>(), 0x16);
        for i in 0..22 {
            assert_eq!(
                buffer[(16 + i * 8)..(16 + (i + 1) * 8)].load_le::<u8>(),
                (i + 1) as u8
            );
        }
    }
}

pub const EL6021_VENDOR_ID: u32 = 2;
pub const EL6021_PRODUCT_ID: u32 = 0x17853052;

pub const EL6021_REVISION_A: u32 = 0x150000;
pub const EL6021_REVISION_B: u32 = 0x140000;
pub const EL6021_REVISION_C: u32 = 0x160000;
pub const EL6021_REVISION_D: u32 = 0x100000;

pub const EL6021_IDENTITY_A: SubDeviceIdentityTuple =
    (EL6021_VENDOR_ID, EL6021_PRODUCT_ID, EL6021_REVISION_A);

pub const EL6021_IDENTITY_B: SubDeviceIdentityTuple =
    (EL6021_VENDOR_ID, EL6021_PRODUCT_ID, EL6021_REVISION_B);

pub const EL6021_IDENTITY_C: SubDeviceIdentityTuple =
    (EL6021_VENDOR_ID, EL6021_PRODUCT_ID, EL6021_REVISION_C);

pub const EL6021_IDENTITY_D: SubDeviceIdentityTuple =
    (EL6021_VENDOR_ID, EL6021_PRODUCT_ID, EL6021_REVISION_D);
