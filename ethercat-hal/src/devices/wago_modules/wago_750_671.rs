/*
* Wago Stepper Controller
* 24 VDC
* 1.5 A
*/

use bitvec::field::BitField;

use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
    EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
};

#[derive(Clone)]
pub struct Wago750_671 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub rxpdo: Wago750_671RxPdo,
    pub txpdo: Wago750_671TxPdo,
    module: Option<Module>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wago750_671OperationMode {
    /// Jogging
    Joggin = 0,
    /// Move program
    MoveProgram = 1,
    /// Positioning
    Positioning = 2,
    /// Velocity
    Velocity = 3,
}

#[derive(Debug, Clone)]
pub struct StmFeatures {
    /// # 0x8012:01
    /// Operation mode
    /// - '0' = Jogging
    /// - '1' = Move program
    /// - '2' = Positioning Controller
    /// - '3' = Velocity
    pub operation_mode: Wago750_671OperationMode,
}

#[derive(Debug, Clone)]
pub struct StmMotorConfiguration {}

#[derive(Clone)]
pub struct Wago750_671Configuration {}

#[derive(Debug, Clone)]
pub enum Wago750_671StepperPort {
    STM1,
}

impl From<Wago750_671StepperPort> for usize {
    fn from(value: Wago750_671StepperPort) -> Self {
        match value {
            Wago750_671StepperPort::STM1 => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Wago750_671DigitalInputPort {
    DI1,
    DI2,
}

impl From<Wago750_671DigitalInputPort> for usize {
    fn from(value: Wago750_671DigitalInputPort) -> Self {
        match value {
            Wago750_671DigitalInputPort::DI1 => 0,
            Wago750_671DigitalInputPort::DI2 => 1,
        }
    }
}

#[derive(Clone, Default)]
pub struct Wago750_671RxPdo {
    pub b: [u8; 12],
}

#[derive(Clone, Default)]
pub struct Wago750_671TxPdo {
    pub b: [u8; 12],
}

impl EthercatDeviceUsed for Wago750_671 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl DynamicEthercatDevice for Wago750_671 {}

impl EthercatDynamicPDO for Wago750_671 {
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

const OFF_C1: usize = 11;

const C1_ENABLE: u8 = 1 << 0;
const C1_STOP2_N: u8 = 1 << 1;
const C1_START: u8 = 1 << 2;
const C1_M_POSITIONING: u8 = 1 << 3;
const C1_M_REFERENCE: u8 = 1 << 5;

const OFF_C3: usize = 9;
const C3_RESET_QUIT: u8 = 1 << 7;

impl Wago750_671 {
    pub fn cmd_reference(&mut self, start: bool) {
        let mut c1 = 0u8;
        c1 |= C1_ENABLE;
        c1 |= C1_STOP2_N;
        c1 |= C1_M_REFERENCE;
        if start {
            c1 |= C1_START;
        }
        self.rxpdo.b[11] = c1;
    }

    pub fn cmd_enable_pos(&mut self, start_pulse: bool) {
        let mut c1 = 0u8;

        // mandatory for any motion
        c1 |= C1_ENABLE;
        c1 |= C1_STOP2_N;

        // select Positioning mode
        c1 |= C1_M_POSITIONING;

        // Start is edge-triggered
        if start_pulse {
            c1 |= C1_START;
        }

        self.rxpdo.b[OFF_C1] = c1;
    }

    pub fn set_positioning_setpoints(&mut self, vel: u16, acc: u16, pos24: i32) {
        // Velocity D0/D1 at offsets 2/3
        let v = vel.to_le_bytes();
        self.rxpdo.b[2] = v[0];
        self.rxpdo.b[3] = v[1];

        // Acceleration D2/D3 at offsets 4/5
        let a = acc.to_le_bytes();
        self.rxpdo.b[4] = a[0];
        self.rxpdo.b[5] = a[1];

        // Target position D4/D5/D6 at offsets 6/7/8 (signed 24-bit)
        let pos = pos24.clamp(-(1 << 23), (1 << 23) - 1) as i32;
        let u = (pos & 0x00FF_FFFF) as u32;
        self.rxpdo.b[6] = (u & 0xFF) as u8;
        self.rxpdo.b[7] = ((u >> 8) & 0xFF) as u8;
        self.rxpdo.b[8] = ((u >> 16) & 0xFF) as u8;
    }

    pub fn pulse_reset_quit(&mut self, pulse: bool) {
        if pulse {
            self.rxpdo.b[OFF_C3] |= C3_RESET_QUIT;
        } else {
            self.rxpdo.b[OFF_C3] &= !C3_RESET_QUIT;
        }
    }
}

impl EthercatDevice for Wago750_671 {
    /*
        Receiving bitslice of the current subdevice in our Loop
    */
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = self.tx_bit_offset;

        for byte_i in 0..12 {
            let bits = &input[base + byte_i * 8..base + (byte_i + 1) * 8];
            self.txpdo.b[byte_i] = bits.load_le::<u8>();
        }

        // TEMP Debug START
        // println!("Wago750_671 IN: {:02X?} \n", self.txpdo.b);

        let s0 = self.txpdo.b[0];
        let s3 = self.txpdo.b[9];
        let s2 = self.txpdo.b[10];
        let s1 = self.txpdo.b[11];

        // println!("S0={:08b} S1={:08b} S2={:08b} S3={:08b}", s0, s1, s2, s3);

        // TEMP Debug END

        Ok(())
    }

    fn input_len(&self) -> usize {
        12 * 8
    }

    fn output(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = self.rx_bit_offset;
        println!("Offset: {}", base);

        for byte_i in 0..12 {
            let bits = &mut output[base + byte_i * 8..base + (byte_i + 1) * 8];
            bits.store_le(self.rxpdo.b[byte_i]);
        }

        Ok(())
    }

    fn output_len(&self) -> usize {
        12 * 8
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
        // validate input has correct length
        let expected = self.input_len();
        let actual = input.len();
        if actual != expected {
            return Err(anyhow::anyhow!(
                "[{}::Device::input_checked] Input length is {} ({} bytes) and must be {} bits ({} bytes)",
                module_path!(),
                actual,
                actual / 8,
                expected,
                expected / 8
            ));
        }
        Ok(())
    }

    fn output_checked(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        // validate output has correct length
        let expected = self.output_len();
        let actual = output.len();
        if actual != expected {
            return Err(anyhow::anyhow!(
                "[{}::Device::input_checked] Input length is {} ({} bytes) and must be {} bits ({} bytes)",
                module_path!(),
                actual,
                actual / 8,
                expected,
                expected / 8
            ));
        }
        Ok(())
    }

    fn get_module(&self) -> Option<crate::devices::Module> {
        self.module.clone()
    }

    fn set_module(&mut self, module: crate::devices::Module) {
        self.tx_bit_offset = module.tx_offset;
        self.rx_bit_offset = module.rx_offset;
        self.module = Some(module)
    }
}

impl EthercatDeviceProcessing for Wago750_671 {}

impl NewEthercatDevice for Wago750_671 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            rxpdo: Wago750_671RxPdo::default(),
            txpdo: Wago750_671TxPdo::default(),
        }
    }
}

impl std::fmt::Debug for Wago750_671 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_671")
    }
}

pub const WAGO_750_671_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_671_PRODUCT_ID: u32 = 108074216;
pub const WAGO_750_671_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_671_VENDOR_ID, WAGO_750_671_PRODUCT_ID);
