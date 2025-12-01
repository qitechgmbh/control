use crate::devices::{EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed, NewEthercatDevice, SubDeviceProductTuple};

#[derive(Clone)]
pub struct Wago750_1506 {
    is_used: bool,
    tx_bit_offset : Option<usize>,
    rx_bit_offset : Option<usize>,
}


impl EthercatDeviceUsed for Wago750_1506 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl EthercatDevice for Wago750_1506 {
	/*
		OK so with ethercrab we receive the bitslice of the current subdevice in our Loop
	*/
    fn input(&mut self, _input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn input_len(&self) -> usize {
        todo!()
    }

    fn output(&self, _output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn output_len(&self) -> usize {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn input_checked(&mut self, input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>) -> Result<(), anyhow::Error> {
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
        self.input(input)
    }

    fn output_checked(&self, output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>) -> Result<(), anyhow::Error> {
        self.output(output)?;

        // validate input has correct length
        let expected = self.output_len();
        let actual = output.len();
        if output.len() != expected {
            return Err(anyhow::anyhow!(
                "[{}::Device::output_checked] Output length is {} ({} bytes) and must be {} bits ({} bytes)",
                module_path!(),
                actual,
                actual / 8,
                expected,
                expected / 8
            ));
        }

        Ok(())
    }
}


impl EthercatDeviceProcessing for Wago750_1506 {}

impl NewEthercatDevice for Wago750_1506 {
    fn new() -> Self {
        Self { is_used: false, tx_bit_offset: None, rx_bit_offset: None }
    }
}

impl std::fmt::Debug for Wago750_1506 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_1506")
    }
}


pub const WAGO_750_1506_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_1506_PRODUCT_ID: u32 = 2147483779;
pub const WAGO_750_1506_MODULE_IDENT :SubDeviceProductTuple  = (WAGO_750_1506_VENDOR_ID,WAGO_750_1506_PRODUCT_ID);