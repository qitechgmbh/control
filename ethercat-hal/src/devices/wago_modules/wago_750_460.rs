use bitvec::field::BitField;

use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
    EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
};
use crate::io::temperature_input::{TemperatureInputDevice, TemperatureInputInput};
use crate::pdo::basic::Limit;

// =============================================================================
// WAGO 750-460 — 4-channel analog input for Pt1000/RTD resistance sensors
// =============================================================================
//
// PDO format (per channel, 16 bits):
//   Bits [15:0] — 16-bit signed measurement value, 0.1 °C per LSB
//                 Valid range : -2000 (= -200.0 °C) … +8500 (= +850.0 °C)
//                 Open circuit: module saturates at +8500 (= +850.0 °C)
//                 Short circuit: module saturates at -2000 (= -200.0 °C)
//
// There are NO status bits packed into the low bits of the data word.
// Wire-break / overrange detection is done by checking whether the value
// is at or beyond the documented measurement limits.
//
// TX PDO: 4 × 16 bits = 64 bits  (16-bit-only mode, no optional status bytes)
// RX PDO: 0 bits
// =============================================================================

#[derive(Debug, Clone, Copy)]
pub enum Wago750_460Port {
    T1,
    T2,
    T3,
    T4,
}

#[derive(Clone, Default)]
pub struct Wago750_460TxPdo {
    t1: u16,
    t2: u16,
    t3: u16,
    t4: u16,
}

impl TemperatureInputDevice<Wago750_460Port> for Wago750_460 {
    fn get_input(&self, port: Wago750_460Port) -> TemperatureInputInput {
        let raw = match port {
            Wago750_460Port::T1 => self.tx_pdo.t1,
            Wago750_460Port::T2 => self.tx_pdo.t2,
            Wago750_460Port::T3 => self.tx_pdo.t3,
            Wago750_460Port::T4 => self.tx_pdo.t4,
        };
        // Full 16-bit signed value, 0.1 °C per LSB — no status bits in the word.
        let temp_raw = raw as i16;
        let temperature = temp_raw as f32 / 10.0;
        let overrange = temp_raw >= 8500;
        let underrange = temp_raw <= -2000;
        TemperatureInputInput {
            temperature,
            undervoltage: underrange,
            overvoltage: overrange,
            limit1: Limit::NotActive,
            limit2: Limit::NotActive,
            error: overrange || underrange,
            txpdo_state: false,
            txpdo_toggle: false,
        }
    }
}

#[derive(Clone)]
pub struct Wago750_460 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    module: Option<Module>,
    tx_pdo: Wago750_460TxPdo,
}

impl DynamicEthercatDevice for Wago750_460 {}

impl EthercatDynamicPDO for Wago750_460 {
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

impl EthercatDeviceUsed for Wago750_460 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl EthercatDevice for Wago750_460 {
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = self.tx_bit_offset;
        self.tx_pdo.t1 = input[base..(base + 16)].load_le::<u16>();
        self.tx_pdo.t2 = input[(base + 16)..(base + 32)].load_le::<u16>();
        self.tx_pdo.t3 = input[(base + 32)..(base + 48)].load_le::<u16>();
        self.tx_pdo.t4 = input[(base + 48)..(base + 64)].load_le::<u16>();
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
        self.module = Some(module);
    }
}

impl EthercatDeviceProcessing for Wago750_460 {}

impl NewEthercatDevice for Wago750_460 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            tx_pdo: Wago750_460TxPdo::default(),
        }
    }
}

impl std::fmt::Debug for Wago750_460 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_460")
    }
}

// =============================================================================
// Device Identity
// =============================================================================

pub const WAGO_750_460_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_460_PRODUCT_ID: u32 = 0x046041b3;
pub const WAGO_750_460_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_460_VENDOR_ID, WAGO_750_460_PRODUCT_ID);
