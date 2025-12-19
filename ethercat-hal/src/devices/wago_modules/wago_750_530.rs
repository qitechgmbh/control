use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
    EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
};
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};

#[derive(Debug, Clone)]
pub enum Wago750_530Port {
    DO1,
    DO2,
    DO3,
    DO4,
    DO5,
    DO6,
    DO7,
    DO8,
}

impl From<Wago750_530Port> for usize {
    fn from(value: Wago750_530Port) -> Self {
        match value {
            Wago750_530Port::DO1 => 0,
            Wago750_530Port::DO2 => 1,
            Wago750_530Port::DO3 => 2,
            Wago750_530Port::DO4 => 3,
            Wago750_530Port::DO5 => 4,
            Wago750_530Port::DO6 => 5,
            Wago750_530Port::DO7 => 6,
            Wago750_530Port::DO8 => 7,
        }
    }
}

#[derive(Clone, Default)]
pub struct Wago750_530RxPdo {
    pub port1: bool,
    pub port2: bool,
    pub port3: bool,
    pub port4: bool,
    pub port5: bool,
    pub port6: bool,
    pub port7: bool,
    pub port8: bool,
}

#[derive(Clone)]
pub struct Wago750_530 {
    is_used: bool,
    tx_bit_offset: usize, 
    rx_bit_offset: usize,
    rxpdo: Wago750_530RxPdo,
    module: Option<Module>,
}


impl DynamicEthercatDevice for Wago750_530 {}

impl EthercatDeviceUsed for Wago750_530 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl EthercatDynamicPDO for Wago750_530 {
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

impl DigitalOutputDevice<Wago750_530Port> for Wago750_530 {
    fn set_output(&mut self, port: Wago750_530Port, value: DigitalOutputOutput) {
        let v: bool = value.into();
        match port {
            Wago750_530Port::DO1 => self.rxpdo.port1 = v,
            Wago750_530Port::DO2 => self.rxpdo.port2 = v,
            Wago750_530Port::DO3 => self.rxpdo.port3 = v,
            Wago750_530Port::DO4 => self.rxpdo.port4 = v,
            Wago750_530Port::DO5 => self.rxpdo.port5 = v,
            Wago750_530Port::DO6 => self.rxpdo.port6 = v,
            Wago750_530Port::DO7 => self.rxpdo.port7 = v,
            Wago750_530Port::DO8 => self.rxpdo.port8 = v,
        }
    }

    fn get_output(&self, port: Wago750_530Port) -> DigitalOutputOutput {
        DigitalOutputOutput(match port {
            Wago750_530Port::DO1 => self.rxpdo.port1,
            Wago750_530Port::DO2 => self.rxpdo.port2,
            Wago750_530Port::DO3 => self.rxpdo.port3,
            Wago750_530Port::DO4 => self.rxpdo.port4,
            Wago750_530Port::DO5 => self.rxpdo.port5,
            Wago750_530Port::DO6 => self.rxpdo.port6,
            Wago750_530Port::DO7 => self.rxpdo.port7,
            Wago750_530Port::DO8 => self.rxpdo.port8,
        })
    }
}

impl EthercatDevice for Wago750_530 {
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
        output.set(self.rx_bit_offset + 0, self.rxpdo.port1);
        output.set(self.rx_bit_offset + 1, self.rxpdo.port2);
        output.set(self.rx_bit_offset + 2, self.rxpdo.port3);
        output.set(self.rx_bit_offset + 3, self.rxpdo.port4);
        output.set(self.rx_bit_offset + 4, self.rxpdo.port5);
        output.set(self.rx_bit_offset + 5, self.rxpdo.port6);
        output.set(self.rx_bit_offset + 6, self.rxpdo.port7);
        output.set(self.rx_bit_offset + 7, self.rxpdo.port8);
        Ok(())
    }

    fn output_len(&self) -> usize {
        8
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

    fn get_module(&self) -> Option<Module> {
        self.module.clone()
    }

    fn set_module(&mut self, module: Module) {
        self.tx_bit_offset = module.tx_offset;
        self.rx_bit_offset = module.rx_offset;
        self.module = Some(module);
    }
}

impl EthercatDeviceProcessing for Wago750_530 {}

impl NewEthercatDevice for Wago750_530 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            rxpdo: Wago750_530RxPdo::default(),
            module: None,
        }
    }
}

impl std::fmt::Debug for Wago750_530 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_530")
    }
}

pub const WAGO_750_530_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_530_PRODUCT_ID: u32 = 2147483778;
pub const WAGO_750_530_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_530_VENDOR_ID, WAGO_750_530_PRODUCT_ID);
