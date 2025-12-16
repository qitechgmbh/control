use crate::{
    devices::{
        DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
        EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
    },
    io::{
        digital_input::{DigitalInputDevice, DigitalInputInput},
        digital_output::{DigitalOutputDevice, DigitalOutputOutput},
    },
};

#[derive(Debug, Clone)]
pub enum Wago750_1506OutputPort {
    DO1,
    DO2,
    DO3,
    DO4,
    DO5,
    DO6,
    DO7,
    DO8,
}

impl From<Wago750_1506OutputPort> for usize {
    fn from(value: Wago750_1506OutputPort) -> Self {
        match value {
            Wago750_1506OutputPort::DO1 => 0,
            Wago750_1506OutputPort::DO2 => 1,
            Wago750_1506OutputPort::DO3 => 2,
            Wago750_1506OutputPort::DO4 => 3,
            Wago750_1506OutputPort::DO5 => 4,
            Wago750_1506OutputPort::DO6 => 5,
            Wago750_1506OutputPort::DO7 => 6,
            Wago750_1506OutputPort::DO8 => 7,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Wago750_1506InputPort {
    DO1,
    DO2,
    DO3,
    DO4,
    DO5,
    DO6,
    DO7,
    DO8,
}

impl From<Wago750_1506InputPort> for usize {
    fn from(value: Wago750_1506InputPort) -> Self {
        match value {
            Wago750_1506InputPort::DO1 => 0,
            Wago750_1506InputPort::DO2 => 1,
            Wago750_1506InputPort::DO3 => 2,
            Wago750_1506InputPort::DO4 => 3,
            Wago750_1506InputPort::DO5 => 4,
            Wago750_1506InputPort::DO6 => 5,
            Wago750_1506InputPort::DO7 => 6,
            Wago750_1506InputPort::DO8 => 7,
        }
    }
}

#[derive(Clone, Default)]
pub struct Wago750_1506RxPdo {
    port1: bool,
    port2: bool,
    port3: bool,
    port4: bool,
    port5: bool,
    port6: bool,
    port7: bool,
    port8: bool,
}

#[derive(Clone, Default)]
pub struct Wago750_1506TxPdo {
    port1: bool,
    port2: bool,
    port3: bool,
    port4: bool,
    port5: bool,
    port6: bool,
    port7: bool,
    port8: bool,
}

impl DigitalInputDevice<Wago750_1506InputPort> for Wago750_1506 {
    fn get_input(&self, port: Wago750_1506InputPort) -> Result<DigitalInputInput, anyhow::Error> {
        Ok(DigitalInputInput {
            value: match port {
                Wago750_1506InputPort::DO1 => self.tx_pdo.port1,
                Wago750_1506InputPort::DO2 => self.tx_pdo.port2,
                Wago750_1506InputPort::DO3 => self.tx_pdo.port3,
                Wago750_1506InputPort::DO4 => self.tx_pdo.port4,
                Wago750_1506InputPort::DO5 => self.tx_pdo.port5,
                Wago750_1506InputPort::DO6 => self.tx_pdo.port6,
                Wago750_1506InputPort::DO7 => self.tx_pdo.port7,
                Wago750_1506InputPort::DO8 => self.tx_pdo.port8,
            },
        })
    }
}

impl DigitalOutputDevice<Wago750_1506OutputPort> for Wago750_1506 {
    /// Writes the new output value into the device's RXPDO structure (in-memory PDI).
    fn set_output(&mut self, port: Wago750_1506OutputPort, value: DigitalOutputOutput) {
        // The DigitalOutputOutput is converted to a bool using the From trait
        let output_value: bool = value.into();
        match port {
            Wago750_1506OutputPort::DO1 => self.rx_pdo.port1 = output_value,
            Wago750_1506OutputPort::DO2 => self.rx_pdo.port2 = output_value,
            Wago750_1506OutputPort::DO3 => self.rx_pdo.port3 = output_value,
            Wago750_1506OutputPort::DO4 => self.rx_pdo.port4 = output_value,
            Wago750_1506OutputPort::DO5 => self.rx_pdo.port5 = output_value,
            Wago750_1506OutputPort::DO6 => self.rx_pdo.port6 = output_value,
            Wago750_1506OutputPort::DO7 => self.rx_pdo.port7 = output_value,
            Wago750_1506OutputPort::DO8 => self.rx_pdo.port8 = output_value,
        }
    }

    /// Reads the current output value from the device's RXPDO structure (in-memory PDI).
    fn get_output(&self, port: Wago750_1506OutputPort) -> DigitalOutputOutput {
        let current_value = match port {
            Wago750_1506OutputPort::DO1 => self.rx_pdo.port1,
            Wago750_1506OutputPort::DO2 => self.rx_pdo.port2,
            Wago750_1506OutputPort::DO3 => self.rx_pdo.port3,
            Wago750_1506OutputPort::DO4 => self.rx_pdo.port4,
            Wago750_1506OutputPort::DO5 => self.rx_pdo.port5,
            Wago750_1506OutputPort::DO6 => self.rx_pdo.port6,
            Wago750_1506OutputPort::DO7 => self.rx_pdo.port7,
            Wago750_1506OutputPort::DO8 => self.rx_pdo.port8,
        };
        // Wrap the bool back into the type-safe wrapper
        DigitalOutputOutput(current_value)
    }
}

#[derive(Clone)]
pub struct Wago750_1506 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    // Should always have on but not when calling constructor
    module: Option<Module>,
    rx_pdo: Wago750_1506RxPdo,
    tx_pdo: Wago750_1506TxPdo,
}

impl DynamicEthercatDevice for Wago750_1506 {}

impl EthercatDynamicPDO for Wago750_1506 {
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

impl EthercatDeviceUsed for Wago750_1506 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl Wago750_1506 {}

impl EthercatDevice for Wago750_1506 {
    // input_len 48 offset: 40
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base = self.tx_bit_offset;

        let idx1 = base + Into::<usize>::into(Wago750_1506InputPort::DO1);
        self.tx_pdo.port1 = *input.get(idx1).expect("Bit 1 out of bounds");

        let idx2 = base + Into::<usize>::into(Wago750_1506InputPort::DO2);
        self.tx_pdo.port2 = *input.get(idx2).expect("Bit 2 out of bounds");

        let idx3 = base + Into::<usize>::into(Wago750_1506InputPort::DO3);
        self.tx_pdo.port3 = *input.get(idx3).expect("Bit 3 out of bounds");

        let idx4 = base + Into::<usize>::into(Wago750_1506InputPort::DO4);
        self.tx_pdo.port4 = *input.get(idx4).expect("Bit 4 out of bounds");

        let idx5 = base + Into::<usize>::into(Wago750_1506InputPort::DO5);
        self.tx_pdo.port5 = *input.get(idx5).expect("Bit 5 out of bounds");

        let idx6 = base + Into::<usize>::into(Wago750_1506InputPort::DO6);
        self.tx_pdo.port6 = *input.get(idx6).expect("Bit 6 out of bounds");

        let idx7 = base + Into::<usize>::into(Wago750_1506InputPort::DO7);
        self.tx_pdo.port7 = *input.get(idx7).expect("Bit 7 out of bounds");

        let idx8 = base + Into::<usize>::into(Wago750_1506InputPort::DO8);
        self.tx_pdo.port8 = *input.get(idx8).expect("Bit 8 out of bounds");
        Ok(())
    }

    fn input_len(&self) -> usize {
        8
    }

    fn output(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        //  println!("output: {} {}",self.rx_bit_offset,_output);
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_1506OutputPort::DO1),
            self.rx_pdo.port1,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_1506OutputPort::DO2),
            self.rx_pdo.port2,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_1506OutputPort::DO3),
            self.rx_pdo.port3,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_1506OutputPort::DO4),
            self.rx_pdo.port4,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_1506OutputPort::DO5),
            self.rx_pdo.port5,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_1506OutputPort::DO6),
            self.rx_pdo.port6,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_1506OutputPort::DO7),
            self.rx_pdo.port7,
        );
        output.set(
            self.rx_bit_offset + Into::<usize>::into(Wago750_1506OutputPort::DO8),
            self.rx_pdo.port8,
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

    fn get_module(&self) -> Option<Module> {
        self.module.clone()
    }

    fn set_module(&mut self, module: Module) {
        self.tx_bit_offset = module.tx_offset;
        self.rx_bit_offset = module.rx_offset;
        self.module = Some(module);
    }
}

impl EthercatDeviceProcessing for Wago750_1506 {
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn output_pre_process(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

impl NewEthercatDevice for Wago750_1506 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            rx_pdo: Wago750_1506RxPdo::default(),
            tx_pdo: Wago750_1506TxPdo::default(),
        }
    }
}

impl std::fmt::Debug for Wago750_1506 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_1506")
    }
}

pub const WAGO_750_1506_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_1506_PRODUCT_ID: u32 = 2147483779;
pub const WAGO_750_1506_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_1506_VENDOR_ID, WAGO_750_1506_PRODUCT_ID);
