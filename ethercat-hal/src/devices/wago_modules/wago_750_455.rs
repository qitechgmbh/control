use bitvec::field::BitField;

use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
    EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
};
use crate::io::analog_input::{AnalogInputDevice, AnalogInputInput};
use crate::io::analog_input::physical::AnalogInputRange;
use units::electric_current::milliampere;
use units::f64::ElectricCurrent;

#[derive(Clone, Debug)]
pub enum Wago750_455Port {
    AI1,
    AI2,
    AI3,
    AI4,
}

impl From<Wago750_455Port> for usize {
    fn from(value: Wago750_455Port) -> Self {
        match value {
            Wago750_455Port::AI1 => 0,
            Wago750_455Port::AI2 => 16,
            Wago750_455Port::AI3 => 32,
            Wago750_455Port::AI4 => 48,
        }
    }
}

#[derive(Clone, Default)]
pub struct Wago750_455TxPdo {
    pub ai1: u16,
    pub ai2: u16,
    pub ai3: u16,
    pub ai4: u16,
}

#[derive(Clone)]
pub struct Wago750_455 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    module: Option<Module>,
    tx_pdo: Wago750_455TxPdo,
}

impl AnalogInputDevice<Wago750_455Port> for Wago750_455 {
    fn get_input(&self, port: Wago750_455Port) -> AnalogInputInput {
        let raw = match port {
            Wago750_455Port::AI1 => self.tx_pdo.ai1,
            Wago750_455Port::AI2 => self.tx_pdo.ai2,
            Wago750_455Port::AI3 => self.tx_pdo.ai3,
            Wago750_455Port::AI4 => self.tx_pdo.ai4,
        };
        let wiring_error = (raw & 0x0003) == 0x0003;
        let raw_value = (raw & 0x7FF0) as i16;
        let normalized = self.analog_input_range().raw_to_normalized(raw_value) as f32;
        AnalogInputInput {
            normalized,
            wiring_error,
        }
    }

    fn analog_input_range(&self) -> AnalogInputRange {
        AnalogInputRange::Current {
            min: ElectricCurrent::new::<milliampere>(4.0),
            max: ElectricCurrent::new::<milliampere>(20.0),
            min_raw: 0,
            max_raw: 0x7FF0,
        }
    }
}

impl EthercatDeviceUsed for Wago750_455 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl DynamicEthercatDevice for Wago750_455 {}

impl EthercatDynamicPDO for Wago750_455 {
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

impl EthercatDevice for Wago750_455 {
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = self.tx_bit_offset;
        let ai1 = input[base..(base + 16)].load_le::<u16>();
        let ai2 = input[(base + 16)..(base + 32)].load_le::<u16>();
        let ai3 = input[(base + 32)..(base + 48)].load_le::<u16>();
        let ai4 = input[(base + 48)..(base + 64)].load_le::<u16>();

        self.tx_pdo.ai1 = ai1;
        self.tx_pdo.ai2 = ai2;
        self.tx_pdo.ai3 = ai3;
        self.tx_pdo.ai4 = ai4;
        Ok(())
    }

    fn input_len(&self) -> usize {
        64
    }

    fn output(
        &self,
        _output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn output_len(&self) -> usize {
        0
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
        _output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn get_module(&self) -> Option<Module> {
        self.module.clone()
    }

    fn set_module(&mut self, module: Module) {
        self.tx_bit_offset = module.tx_offset;
        self.rx_bit_offset = module.rx_offset;
        self.module = Some(module)
    }
}

impl EthercatDeviceProcessing for Wago750_455 {}

impl NewEthercatDevice for Wago750_455 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            tx_pdo: Wago750_455TxPdo::default(),
        }
    }
}

impl std::fmt::Debug for Wago750_455 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_455")
    }
}

pub const WAGO_750_455_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_455_PRODUCT_ID: u32 = 0x045541b3;
pub const WAGO_750_455_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_455_VENDOR_ID, WAGO_750_455_PRODUCT_ID);
