use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceUsed, EthercatDynamicPDO, Module,
    SubDeviceProductTuple,
};
use crate::devices::{EthercatDeviceProcessing, NewEthercatDevice};
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};

#[derive(Clone)]
pub struct Wago750_501 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub rxpdo: Wago750_501RxPdo,
    module: Option<Module>,
}

#[derive(Debug, Clone)]
pub enum Wago750_501Port {
    Port1,
    Port2,
}

impl From<Wago750_501Port> for usize {
    fn from(value: Wago750_501Port) -> Self {
        match value {
            Wago750_501Port::Port1 => 0,
            Wago750_501Port::Port2 => 1,
        }
    }
}

#[derive(Clone, Default)]
pub struct Wago750_501RxPdo {
    pub port1: bool,
    pub port2: bool,
}

impl DigitalOutputDevice<Wago750_501Port> for Wago750_501 {
    /// Writes the new output value into the device's RXPDO structure (in-memory PDI).
    fn set_output(&mut self, port: Wago750_501Port, value: DigitalOutputOutput) {
        // The DigitalOutputOutput is converted to a bool using the From trait
        let output_value: bool = value.into();
        match port {
            Wago750_501Port::Port1 => {
                // Modify the internal PDO representation
                self.rxpdo.port1 = output_value;
            }
            Wago750_501Port::Port2 => {
                // Modify the internal PDO representation
                self.rxpdo.port2 = output_value;
            }
        }
    }

    /// Reads the current output value from the device's RXPDO structure (in-memory PDI).
    fn get_output(&self, port: Wago750_501Port) -> DigitalOutputOutput {
        let current_value = match port {
            Wago750_501Port::Port1 => self.rxpdo.port1,
            Wago750_501Port::Port2 => self.rxpdo.port2,
        };

        // Wrap the bool back into the type-safe wrapper
        DigitalOutputOutput(current_value)
    }
}

impl EthercatDeviceUsed for Wago750_501 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl DynamicEthercatDevice for Wago750_501 {}

impl EthercatDynamicPDO for Wago750_501 {
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

impl EthercatDevice for Wago750_501 {
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
            self.rx_bit_offset + Into::<usize>::into(Wago750_501Port::Port1),
            self.rxpdo.port1,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_501Port::Port2),
            self.rxpdo.port2,
        );
        Ok(())
    }

    fn output_len(&self) -> usize {
        2
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

impl EthercatDeviceProcessing for Wago750_501 {}

impl NewEthercatDevice for Wago750_501 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            rxpdo: Wago750_501RxPdo::default(),
        }
    }
}

impl std::fmt::Debug for Wago750_501 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_501")
    }
}

pub const WAGO_750_501_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_501_PRODUCT_ID: u32 = 2147483682;
pub const WAGO_750_501_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_501_VENDOR_ID, WAGO_750_501_PRODUCT_ID);
