use bitvec::field::BitField;

use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceUsed, EthercatDynamicPDO, Module,
    SubDeviceProductTuple,
};
use crate::devices::{EthercatDeviceProcessing, NewEthercatDevice};
use crate::io::serial_interface::SerialInterfaceDevice;
const WAGO750_652_MAX_BUF_LENGTH: usize = 48;

#[derive(Clone)]
pub struct Wago750_652 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    module: Option<Module>,
    started_init: bool,
    pub rx_pdo: Wago750_652RxPdo,
    pub tx_pdo: Wago750_652TxPdo,
    has_messages_last_toggle: bool,
}

#[derive(Clone, Default, Debug)]
pub struct Wago750_652Control {
    pub transmit_request: bool, // Transmit request
    pub receive_ack: bool,      // Receive acknowledge
    pub init_request: bool,     // Initialization request
    pub send_continuous: bool,  // Send continuous
    pub output_length: usize,
}

#[derive(Clone, Debug)]
pub struct Wago750_652RxPdo {
    pub control: Wago750_652Control,
    pub out_buffer: [u8; WAGO750_652_MAX_BUF_LENGTH],
}

impl Default for Wago750_652RxPdo {
    fn default() -> Self {
        Self {
            control: Default::default(),
            out_buffer: [0u8; WAGO750_652_MAX_BUF_LENGTH],
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Wago750_652Status {
    pub transmit_ack: bool, //
    pub receive_request: bool,
    pub init_ack: bool,
    pub buf_full: bool,
    pub buf_empty: bool, // Buffer is empty (Sending inactive)
    pub error_parity: bool,
    pub error_framing: bool, // Frame is corrupt?
    pub error_overrun: bool, // Byte/s was lost on receive
    pub input_length: usize,
}

#[derive(Clone, Debug)]
pub struct Wago750_652TxPdo {
    pub status: Wago750_652Status,
    pub in_buffer: [u8; WAGO750_652_MAX_BUF_LENGTH],
}

impl Default for Wago750_652TxPdo {
    fn default() -> Self {
        Self {
            status: Default::default(),
            in_buffer: [0u8; WAGO750_652_MAX_BUF_LENGTH],
        }
    }
}

impl EthercatDeviceUsed for Wago750_652 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl DynamicEthercatDevice for Wago750_652 {}
impl EthercatDynamicPDO for Wago750_652 {
    fn get_tx_offset(&self) -> usize {
        self.tx_bit_offset
    }

    fn get_rx_offset(&self) -> usize {
        self.rx_bit_offset
    }

    fn set_tx_offset(&mut self, offset: usize) {
        self.tx_bit_offset = offset
    }

    fn set_rx_offset(&mut self, offset: usize) {
        self.rx_bit_offset = offset
    }
}

impl EthercatDevice for Wago750_652 {
    /*
        OK so with ethercrab we receive the bitslice of the current subdevice in our Loop
    */
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        self.tx_pdo.in_buffer.fill(0u8);
        let base = self.tx_bit_offset;

        self.tx_pdo.status.transmit_ack = *input.get(base + 0).expect("Missing buf_empty bit");
        self.tx_pdo.status.receive_request = *input.get(base + 1).expect("Missing buf_empty bit");
        self.tx_pdo.status.init_ack = *input.get(base + 2).expect("Missing buf_empty bit");
        self.tx_pdo.status.buf_full = *input.get(base + 6).expect("Missing buf_empty bit");
        self.tx_pdo.status.buf_empty = *input.get(base + 11).expect("Missing buf_empty bit");
        self.tx_pdo.status.error_parity = *input.get(base + 13).expect("Missing buf_empty bit");
        self.tx_pdo.status.error_framing = *input.get(base + 14).expect("Missing buf_empty bit");
        self.tx_pdo.status.error_overrun = *input.get(base + 15).expect("Missing buf_empty bit");
        // Im sorry but this is what you get by splitting the len across different bytes ...
        self.tx_pdo.status.input_length = ((*input.get(base + 3).expect("Missing buf_empty bit")
            as u8)
            ^ ((*input.get(base + 4).expect("Missing buf_empty bit") as u8) << 1)
            ^ ((*input.get(base + 5).expect("Missing buf_empty bit") as u8) << 2)
            ^ ((*input.get(base + 8).expect("Missing buf_empty bit") as u8) << 3)
            ^ ((*input.get(base + 9).expect("Missing buf_empty bit") as u8) << 4)
            ^ ((*input.get(base + 10).expect("Missing buf_empty bit") as u8) << 5))
            as usize;

        // the serial bytes start at bit 16,
        // then we calculate the offset of when the serial bytes buffer is over, which for the 24byte pdo is 22bytes
        // to get the size in bits we do 22 * 8 -> 176bits
        let serial_bytes = input[(base + 16)..((base + 16) + 22 * 8_usize)].chunks_exact(8);
        for (i, val) in serial_bytes.enumerate() {
            self.tx_pdo.in_buffer[i] = val.load_le();
        }
        Ok(())
    }

    fn input_len(&self) -> usize {
        24
    }

    fn output(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = 32;

        let mut ol0_i = 5; // Ol0 starts at bit 5 and goes to bit 7 (inclusive) 
        let mut ol1_i = 8;

        output.set(base + 0, self.rx_pdo.control.transmit_request);
        output.set(base + 1, self.rx_pdo.control.receive_ack);
        output.set(base + 2, self.rx_pdo.control.init_request);
        output.set(base + 3, self.rx_pdo.control.send_continuous);

        for i in 0..3 {
            // Rust type system proceeds to annoy once again, we need to add a != at the end, because a cast to bool is just too dangeerous obviously
            output.set(
                base + ol0_i,
                ((self.rx_pdo.control.output_length >> i) & 1) != 0,
            );
            ol0_i += 1;
        }

        for i in 3..6 {
            output.set(
                base + ol1_i,
                ((self.rx_pdo.control.output_length >> i) & 1) != 0,
            );
            ol1_i += 1;
        }
        println!("{}", self.rx_pdo.control.output_length);
        let data_start = base + 16;
        for (i, &byte) in self.rx_pdo.out_buffer.iter().take(22).enumerate() {
            output[(data_start + i * 8)..(data_start + (i + 1) * 8)].store_le(byte);
        }
        //println!("{:?}",self.rx_pdo.out_buffer);
        Ok(())
    }

    fn output_len(&self) -> usize {
        24
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn is_module(&self) -> bool {
        true
    }

    fn input_checked(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        self.input(input)
    }

    fn output_checked(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        self.output(output)?;
        Ok(())
    }

    fn get_module(&self) -> Option<Module> {
        self.module.clone()
    }

    fn set_module(&mut self, module: Module) {
        self.tx_bit_offset = module.tx_offset;
        self.rx_bit_offset = module.rx_offset;
        self.module = Some(module);
    }
}

impl EthercatDeviceProcessing for Wago750_652 {}

impl NewEthercatDevice for Wago750_652 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            rx_pdo: Wago750_652RxPdo::default(),
            tx_pdo: Wago750_652TxPdo::default(),
            started_init: false,
            has_messages_last_toggle: false,
        }
    }
}

impl std::fmt::Debug for Wago750_652 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_652")
    }
}

#[derive(Clone)]
pub enum Wago750_652Port {
    SI1, // Serial
}

impl SerialInterfaceDevice<Wago750_652Port> for Wago750_652 {
    fn serial_interface_read_message(&mut self, port: Wago750_652Port) -> Option<Vec<u8>> {
        if !self.serial_interface_has_messages(port) {
            return None;
        } else {
            let in_len = self.tx_pdo.status.input_length;
            let received_data = self.tx_pdo.in_buffer[0..in_len].to_vec();

            if received_data.is_empty() {
                return None;
            }

            self.has_messages_last_toggle = self.tx_pdo.status.receive_request;
            self.rx_pdo.control.receive_ack = !self.rx_pdo.control.receive_ack;

            Some(received_data)
        }
    }

    fn serial_interface_write_message(
        &mut self,
        _port: Wago750_652Port,
        message: Vec<u8>,
    ) -> Result<bool, anyhow::Error> {
        if message.len() > WAGO750_652_MAX_BUF_LENGTH {
            return Err(anyhow::anyhow!(
                "Message is too long for RxPdo Buffer of 22 bytes!"
            ));
        }

        if message.is_empty() {
            return Ok(self.rx_pdo.control.transmit_request == self.tx_pdo.status.transmit_ack);
        }

        let bytes = message.as_slice();
        self.rx_pdo.control.output_length = message.len();
        self.rx_pdo.out_buffer.fill(0u8);
        self.rx_pdo.out_buffer[0..message.len()].copy_from_slice(&bytes[0..message.len()]);
        self.rx_pdo.control.transmit_request = !self.rx_pdo.control.transmit_request;
        Ok(true)
    }

    fn serial_interface_has_messages(&mut self, _port: Wago750_652Port) -> bool {
        return self.tx_pdo.status.receive_request != self.has_messages_last_toggle;
    }

    fn get_serial_encoding(
        &self,
        _port: Wago750_652Port,
    ) -> Option<crate::io::serial_interface::SerialEncoding> {
        None
    }

    fn get_baudrate(&self, _port: Wago750_652Port) -> Option<u32> {
        // Right now we cant change it on the fly because WAGO ONLY supports setting via
        // Codesys OR WAGO I/O Check the way it is set with Codesys is also NOT documented ...
        // My Hunch is that the 0x2010 0x2011 register can be used to configure that with COE its not documented though
        None
    }

    /*
        Needs to be called multiple times until it returns true, at which point it is ready to be used
    */
    fn serial_interface_initialize(&mut self, port: Wago750_652Port) -> bool {
        match port {
            Wago750_652Port::SI1 => {
                if !self.rx_pdo.control.init_request
                    && !self.tx_pdo.status.init_ack
                    && !self.started_init
                {
                    self.rx_pdo.control.init_request = true;
                    self.started_init = true;
                    return false;
                }

                if self.rx_pdo.control.init_request && self.tx_pdo.status.init_ack {
                    self.rx_pdo.control.init_request = false;
                    return false;
                }

                if !self.rx_pdo.control.init_request && self.tx_pdo.status.init_ack {
                    return false;
                }

                if !self.rx_pdo.control.init_request
                    && !self.tx_pdo.status.init_ack
                    && self.started_init
                {
                    self.has_messages_last_toggle = self.tx_pdo.status.receive_request;
                    return true;
                }
                false
            }
        }
    }
}

pub const WAGO_750_652_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_652_PRODUCT_ID: u32 = 106043250;
pub const WAGO_750_652_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_652_VENDOR_ID, WAGO_750_652_PRODUCT_ID);
