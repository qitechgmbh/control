use bitvec::field::BitField;

use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
    EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
};
use crate::io::analog_input::physical::AnalogInputRange;
use crate::io::analog_input::{AnalogInputDevice, AnalogInputInput};
use units::electric_potential::volt;
use units::f64::ElectricPotential;

#[derive(Clone, Debug)]
pub enum Wago750_467Port {
    AI1,
    AI2,
}

impl From<Wago750_467Port> for usize {
    fn from(value: Wago750_467Port) -> Self {
        match value {
            Wago750_467Port::AI1 => 0,
            Wago750_467Port::AI2 => 16,
        }
    }
}

#[derive(Clone, Default)]
pub struct Wago750_467TxPdo {
    pub ai1: u16,
    pub ai2: u16,
}

#[derive(Clone)]
pub struct Wago750_467 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    module: Option<Module>,
    tx_pdo: Wago750_467TxPdo,
}

impl AnalogInputDevice<Wago750_467Port> for Wago750_467 {
    fn get_input(&self, port: Wago750_467Port) -> AnalogInputInput {
        let raw = match port {
            Wago750_467Port::AI1 => self.tx_pdo.ai1,
            Wago750_467Port::AI2 => self.tx_pdo.ai2,
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
        AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(10.0),
            min_raw: 0,
            max_raw: i16::MAX,
        }
    }
}

impl EthercatDeviceUsed for Wago750_467 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl DynamicEthercatDevice for Wago750_467 {}

impl EthercatDynamicPDO for Wago750_467 {
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

impl EthercatDevice for Wago750_467 {
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = self.tx_bit_offset;
        let ai1 = input[base..(base + 16)].load_le::<u16>();
        let ai2 = input[(base + 16)..(base + 32)].load_le::<u16>();

        self.tx_pdo.ai1 = ai1;
        self.tx_pdo.ai2 = ai2;
        Ok(())
    }

    fn input_len(&self) -> usize {
        32
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

impl EthercatDeviceProcessing for Wago750_467 {}

impl NewEthercatDevice for Wago750_467 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            tx_pdo: Wago750_467TxPdo::default(),
        }
    }
}

impl std::fmt::Debug for Wago750_467 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_467")
    }
}

pub const WAGO_750_467_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_467_PRODUCT_ID: u32 = 0x046741ad;
pub const WAGO_750_467_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_467_VENDOR_ID, WAGO_750_467_PRODUCT_ID);
