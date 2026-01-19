use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceUsed, EthercatDynamicPDO, Module,
    SubDeviceProductTuple,
};
use crate::devices::{EthercatDeviceProcessing, NewEthercatDevice};
use crate::io::digital_input::{DigitalInputDevice, DigitalInputInput};

#[derive(Clone)]
pub struct Wago750_430 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub txpdo: Wago750_430TxPdo,
    module: Option<Module>,
}

#[derive(Debug, Clone)]
pub enum Wago750_430Port {
    Port1,
    Port2,
    Port3,
    Port4,
    Port5,
    Port6,
    Port7,
    Port8,
}

impl From<Wago750_430Port> for usize {
    fn from(value: Wago750_430Port) -> Self {
        match value {
            Wago750_430Port::Port1 => 0,
            Wago750_430Port::Port2 => 1,
            Wago750_430Port::Port3 => 2,
            Wago750_430Port::Port4 => 3,
            Wago750_430Port::Port5 => 4,
            Wago750_430Port::Port6 => 5,
            Wago750_430Port::Port7 => 6,
            Wago750_430Port::Port8 => 7,
        }
    }
}

#[derive(Clone, Default)]
pub struct Wago750_430TxPdo {
    pub port1: bool,
    pub port2: bool,
    pub port3: bool,
    pub port4: bool,
    pub port5: bool,
    pub port6: bool,
    pub port7: bool,
    pub port8: bool,
}

impl DigitalInputDevice<Wago750_430Port> for Wago750_430 {
    fn get_input(
        &self,
        port: Wago750_430Port,
    ) -> Result<crate::io::digital_input::DigitalInputInput, anyhow::Error> {
        match port {
            Wago750_430Port::Port1 => Ok(DigitalInputInput {
                value: self.txpdo.port1,
            }),
            Wago750_430Port::Port2 => Ok(DigitalInputInput {
                value: self.txpdo.port2,
            }),
            Wago750_430Port::Port3 => Ok(DigitalInputInput {
                value: self.txpdo.port3,
            }),
            Wago750_430Port::Port4 => Ok(DigitalInputInput {
                value: self.txpdo.port4,
            }),
            Wago750_430Port::Port5 => Ok(DigitalInputInput {
                value: self.txpdo.port5,
            }),
            Wago750_430Port::Port6 => Ok(DigitalInputInput {
                value: self.txpdo.port6,
            }),
            Wago750_430Port::Port7 => Ok(DigitalInputInput {
                value: self.txpdo.port7,
            }),
            Wago750_430Port::Port8 => Ok(DigitalInputInput {
                value: self.txpdo.port8,
            }),
        }
    }
}

impl EthercatDeviceUsed for Wago750_430 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl DynamicEthercatDevice for Wago750_430 {}

impl EthercatDynamicPDO for Wago750_430 {
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

impl EthercatDevice for Wago750_430 {
    fn input(
        &mut self,
        _input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn input_len(&self) -> usize {
        0
    }

    fn output(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
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
        _input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn output_checked(
        &self,
        _output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
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

impl EthercatDeviceProcessing for Wago750_430 {}

impl NewEthercatDevice for Wago750_430 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            txpdo: Wago750_430TxPdo::default(),
        }
    }
}

impl std::fmt::Debug for Wago750_430 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_430")
    }
}

pub const WAGO_750_430_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_430_PRODUCT_ID: u32 = 2147483777;
pub const WAGO_750_430_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_430_VENDOR_ID, WAGO_750_430_PRODUCT_ID);
