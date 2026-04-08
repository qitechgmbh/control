use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
    EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
};
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};

#[derive(Clone)]
pub struct Wago750_531 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub rx_pdo: Wago750_531RxPdo,
    module: Option<Module>,
}

#[derive(Debug, Clone)]
pub enum Wago750_531OutputPort {
    DO1,
    DO2,
    DO3,
    DO4,
}

impl From<Wago750_531OutputPort> for usize {
    fn from(value: Wago750_531OutputPort) -> Self {
        match value {
            Wago750_531OutputPort::DO1 => 0,
            Wago750_531OutputPort::DO2 => 1,
            Wago750_531OutputPort::DO3 => 2,
            Wago750_531OutputPort::DO4 => 3,
        }
    }
}

#[derive(Clone, Default)]
pub struct Wago750_531RxPdo {
    pub port1: bool,
    pub port2: bool,
    pub port3: bool,
    pub port4: bool,
}

impl DigitalOutputDevice<Wago750_531OutputPort> for Wago750_531 {
    fn set_output(&mut self, port: Wago750_531OutputPort, value: DigitalOutputOutput) {
        let output_value: bool = value.into();
        match port {
            Wago750_531OutputPort::DO1 => self.rx_pdo.port1 = output_value,
            Wago750_531OutputPort::DO2 => self.rx_pdo.port2 = output_value,
            Wago750_531OutputPort::DO3 => self.rx_pdo.port3 = output_value,
            Wago750_531OutputPort::DO4 => self.rx_pdo.port4 = output_value,
        }
    }

    fn get_output(&self, port: Wago750_531OutputPort) -> DigitalOutputOutput {
        let current_value = match port {
            Wago750_531OutputPort::DO1 => self.rx_pdo.port1,
            Wago750_531OutputPort::DO2 => self.rx_pdo.port2,
            Wago750_531OutputPort::DO3 => self.rx_pdo.port3,
            Wago750_531OutputPort::DO4 => self.rx_pdo.port4,
        };
        DigitalOutputOutput(current_value)
    }
}

impl DynamicEthercatDevice for Wago750_531 {}

impl EthercatDynamicPDO for Wago750_531 {
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

impl EthercatDeviceUsed for Wago750_531 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl EthercatDevice for Wago750_531 {
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
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_531OutputPort::DO1),
            self.rx_pdo.port1,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_531OutputPort::DO2),
            self.rx_pdo.port2,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_531OutputPort::DO3),
            self.rx_pdo.port3,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_531OutputPort::DO4),
            self.rx_pdo.port4,
        );
        Ok(())
    }

    fn output_len(&self) -> usize {
        4
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

impl EthercatDeviceProcessing for Wago750_531 {}

impl NewEthercatDevice for Wago750_531 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            rx_pdo: Wago750_531RxPdo::default(),
        }
    }
}

impl std::fmt::Debug for Wago750_531 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_531")
    }
}

pub const WAGO_750_531_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_531_PRODUCT_ID: u32 = 2147483714;
pub const WAGO_750_531_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_531_VENDOR_ID, WAGO_750_531_PRODUCT_ID);
