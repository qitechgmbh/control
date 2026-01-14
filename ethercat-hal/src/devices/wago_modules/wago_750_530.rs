use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceUsed, EthercatDynamicPDO, Module,
    SubDeviceProductTuple,
};
use crate::devices::{EthercatDeviceProcessing, NewEthercatDevice};
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};

#[derive(Clone)]
pub struct Wago750_530 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub rxpdo: Wago750_530RxPdo,
    module: Option<Module>,
}

#[derive(Debug, Clone)]
pub enum Wago750_530Port {
    Port1,
    Port2,
    Port3,
    Port4,
    Port5,
    Port6,
    Port7,
    Port8,
}

impl From<Wago750_530Port> for usize {
    fn from(value: Wago750_530Port) -> Self {
        match value {
            Wago750_530Port::Port1 => 0,
            Wago750_530Port::Port2 => 1,
            Wago750_530Port::Port3 => 2,
            Wago750_530Port::Port4 => 3,
            Wago750_530Port::Port5 => 4,
            Wago750_530Port::Port6 => 5,
            Wago750_530Port::Port7 => 6,
            Wago750_530Port::Port8 => 7,
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

impl DigitalOutputDevice<Wago750_530Port> for Wago750_530 {
    /// Writes the new output value into the device's RXPDO structure (in-memory PDI).
    fn set_output(&mut self, port: Wago750_530Port, value: DigitalOutputOutput) {
        // The DigitalOutputOutput is converted to a bool using the From trait
        let output_value: bool = value.into();
        match port {
            Wago750_530Port::Port1 => {
                // Modify the internal PDO representation
                self.rxpdo.port1 = output_value;
            }
            Wago750_530Port::Port2 => {
                // Modify the internal PDO representation
                self.rxpdo.port2 = output_value;
            }
            Wago750_530Port::Port3 => {
                // Modify the internal PDO representation
                self.rxpdo.port3 = output_value;
            }
            Wago750_530Port::Port4 => {
                // Modify the internal PDO representation
                self.rxpdo.port4 = output_value;
            }
            Wago750_530Port::Port5 => {
                // Modify the internal PDO representation
                self.rxpdo.port5 = output_value;
            }
            Wago750_530Port::Port6 => {
                // Modify the internal PDO representation
                self.rxpdo.port6 = output_value;
            }
            Wago750_530Port::Port7 => {
                // Modify the internal PDO representation
                self.rxpdo.port7 = output_value;
            }
            Wago750_530Port::Port8 => {
                // Modify the internal PDO representation
                self.rxpdo.port8 = output_value;
            }
        }
    }

    /// Reads the current output value from the device's RXPDO structure (in-memory PDI).
    fn get_output(&self, port: Wago750_530Port) -> DigitalOutputOutput {
        let current_value = match port {
            Wago750_530Port::Port1 => self.rxpdo.port1,
            Wago750_530Port::Port2 => self.rxpdo.port2,
            Wago750_530Port::Port3 => self.rxpdo.port3,
            Wago750_530Port::Port4 => self.rxpdo.port4,
            Wago750_530Port::Port5 => self.rxpdo.port5,
            Wago750_530Port::Port6 => self.rxpdo.port6,
            Wago750_530Port::Port7 => self.rxpdo.port7,
            Wago750_530Port::Port8 => self.rxpdo.port8,
        };

        // Wrap the bool back into the type-safe wrapper
        DigitalOutputOutput(current_value)
    }
}

impl EthercatDeviceUsed for Wago750_530 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl DynamicEthercatDevice for Wago750_530 {}

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
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_530Port::Port1),
            self.rxpdo.port1,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_530Port::Port2),
            self.rxpdo.port2,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_530Port::Port3),
            self.rxpdo.port3,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_530Port::Port4),
            self.rxpdo.port4,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_530Port::Port5),
            self.rxpdo.port5,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_530Port::Port6),
            self.rxpdo.port6,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_530Port::Port7),
            self.rxpdo.port7,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_530Port::Port8),
            self.rxpdo.port8,
        );
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

    fn get_module(&self) -> Option<crate::devices::Module> {
        self.module.clone()
    }

    fn set_module(&mut self, module: crate::devices::Module) {
        self.tx_bit_offset = module.tx_offset;
        self.rx_bit_offset = module.rx_offset;
        self.module = Some(module)
    }
}

impl EthercatDeviceProcessing for Wago750_530 {}

impl NewEthercatDevice for Wago750_530 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            rxpdo: Wago750_530RxPdo::default(),
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
