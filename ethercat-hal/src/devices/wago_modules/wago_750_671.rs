/*
* Wago Stepper Controller
* 24 VDC
* 1.5 A
*/

use crate::{devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed, EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple
}, io::digital_input::{DigitalInputDevice, DigitalInputInput}};

#[derive(Clone)]
pub struct Wago750_671 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    pub rxpdo: Wago750_671RxPdo,
    pub txpdo: Wago750_671TxPdo,
    module: Option<Module>,
}

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
}

#[derive(Clone, Default)]
pub struct Wago750_671TxPdo {
    port1: bool,
    port2: bool,
}

impl DigitalInputDevice<Wago750_671DigitalInputPort> for Wago750_671 {
    fn get_input(&self, port: Wago750_671DigitalInputPort) -> Result<DigitalInputInput, anyhow::Error> {
        Ok(DigitalInputInput {
            value: match port {
                Wago750_671DigitalInputPort::DI1 => self.txpdo.port1,
                Wago750_671DigitalInputPort::DI2 => self.txpdo.port2,
            },
        })
    }
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

impl EthercatDevice for Wago750_671 {
    /*
        Receiving bitslice of the current subdevice in our Loop
    */
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
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
