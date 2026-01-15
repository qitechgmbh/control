use crate::{
    devices::{
        DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
        EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
    },
    io::{
        digital_input::{DigitalInputDevice, DigitalInputInput},
    },
};

#[derive(Debug, Clone)]
pub enum Wago750_402InputPort {
    DO1,
    DO2,
    DO3,
    DO4,
}

impl From<Wago750_402InputPort> for usize {
    fn from(value: Wago750_402InputPort) -> Self {
        match value {
            Wago750_402InputPort::DO1 => 0,
            Wago750_402InputPort::DO2 => 1,
            Wago750_402InputPort::DO3 => 2,
            Wago750_402InputPort::DO4 => 3,
        }
    }
}


#[derive(Clone, Default)]
pub struct Wago750_402TxPdo {
    port1: bool,
    port2: bool,
    port3: bool,
    port4: bool,
}

impl DigitalInputDevice<Wago750_402InputPort> for Wago750_402 {
    fn get_input(&self, port: Wago750_402InputPort) -> Result<DigitalInputInput, anyhow::Error> {
        Ok(DigitalInputInput {
            value: match port {
                Wago750_402InputPort::DO1 => self.tx_pdo.port1,
                Wago750_402InputPort::DO2 => self.tx_pdo.port2,
                Wago750_402InputPort::DO3 => self.tx_pdo.port3,
                Wago750_402InputPort::DO4 => self.tx_pdo.port4,
            },
        })
    }
}

#[derive(Clone)]
pub struct Wago750_402 {
    is_used: bool,
    tx_bit_offset: usize,
    // Should always have on but not when calling constructor
    module: Option<Module>,
    tx_pdo: Wago750_402TxPdo,
}

impl DynamicEthercatDevice for Wago750_402 {}

impl EthercatDynamicPDO for Wago750_402 {
    fn get_tx_offset(&self) -> usize {
        self.tx_bit_offset
    }

    fn get_rx_offset(&self) -> usize {
        0 //this device has not rx
    }

    fn set_tx_offset(&mut self, offset: usize) {
        self.tx_bit_offset = offset
    }

    fn set_rx_offset(&mut self, offset: usize) {
        // it does nothing because the device dont have rx
    }
}

impl EthercatDeviceUsed for Wago750_402 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl Wago750_402 {}

impl EthercatDevice for Wago750_402 {
    // input_len 48 offset: 40
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = self.tx_bit_offset;

        let idx1 = base + Into::<usize>::into(Wago750_402InputPort::DO1);
        self.tx_pdo.port1 = *input.get(idx1).expect("Bit 1 out of bounds");

        let idx2 = base + Into::<usize>::into(Wago750_402InputPort::DO2);
        self.tx_pdo.port2 = *input.get(idx2).expect("Bit 2 out of bounds");

        let idx3 = base + Into::<usize>::into(Wago750_402InputPort::DO3);
        self.tx_pdo.port3 = *input.get(idx3).expect("Bit 3 out of bounds");

        let idx4 = base + Into::<usize>::into(Wago750_402InputPort::DO4);
        self.tx_pdo.port4 = *input.get(idx4).expect("Bit 4 out of bounds");
        Ok(())
    }

    fn input_len(&self) -> usize {
        4
    }

    fn output(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        //  println!("output: {} {}",self.rx_bit_offset,_output);
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

    fn get_module(&self) -> Option<Module> {
        self.module.clone()
    }

    fn set_module(&mut self, module: Module) {
        self.tx_bit_offset = module.tx_offset;
        self.module = Some(module);
    }
}

impl EthercatDeviceProcessing for Wago750_402 {
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn output_pre_process(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

impl NewEthercatDevice for Wago750_402 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            module: None,
            tx_pdo: Wago750_402TxPdo::default(),
        }
    }
}

impl std::fmt::Debug for Wago750_402 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_402")
    }
}

pub const WAGO_750_402_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_402_PRODUCT_ID: u32 = 2147483713;
pub const WAGO_750_402_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_402_VENDOR_ID, WAGO_750_402_PRODUCT_ID);
