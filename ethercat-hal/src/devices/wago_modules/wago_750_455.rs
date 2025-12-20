use crate::{
    devices::{
        DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
        EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
    },
    io::analog_input::{physical::AnalogInputRange, AnalogInputDevice, AnalogInputInput},
};
use units::{electric_current::milliampere, f64::ElectricCurrent};

/// Port enum for the 4-channel analog input module
#[derive(Debug, Clone)]
pub enum Wago750_455Port {
    AI1,
    AI2,
    AI3,
    AI4,
}

impl From<Wago750_455Port> for usize {
    fn from(value: Wago750_455Port) -> Self {
        match value {
            Wago750_455Port::AI1 => 0,
            Wago750_455Port::AI2 => 1,
            Wago750_455Port::AI3 => 2,
            Wago750_455Port::AI4 => 3,
        }
    }
}

/// TxPdo structure for the Wago 750-455
/// Contains the 4 analog input values received from the device
#[derive(Clone, Default)]
pub struct Wago750_455TxPdo {
    pub channel1: i16,
    pub channel2: i16,
    pub channel3: i16,
    pub channel4: i16,
}

/// Main device structure for Wago 750-455
#[derive(Clone)]
pub struct Wago750_455 {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    module: Option<Module>,
    pub txpdo: Wago750_455TxPdo,
}

impl AnalogInputDevice<Wago750_455Port> for Wago750_455 {
    fn get_input(&self, port: Wago750_455Port) -> AnalogInputInput {
        let raw_value = match port {
            Wago750_455Port::AI1 => self.txpdo.channel1,
            Wago750_455Port::AI2 => self.txpdo.channel2,
            Wago750_455Port::AI3 => self.txpdo.channel3,
            Wago750_455Port::AI4 => self.txpdo.channel4,
        };

        // Normalize the value to -1.0 to 1.0 range
        // The Wago 750-455 provides 4-20mA input with 16-bit signed resolution
        // Full scale is ±32767
        let normalized = f32::from(raw_value) / f32::from(i16::MAX);

        AnalogInputInput {
            normalized,
            wiring_error: false,
        }
    }

    fn analog_input_range(&self) -> AnalogInputRange {
        // Wago 750-455 is a 4-20mA current input module
        AnalogInputRange::Current {
            min: ElectricCurrent::new::<milliampere>(4.0),
            max: ElectricCurrent::new::<milliampere>(20.0),
            min_raw: 0,
            max_raw: 32767,
        }
    }
}

impl EthercatDeviceUsed for Wago750_455 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl DynamicEthercatDevice for Wago750_455 {}

impl EthercatDynamicPDO for Wago750_455 {
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

impl EthercatDevice for Wago750_455 {
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        let base_bit = self.tx_bit_offset;

        // Each channel is 16 bits (2 bytes)
        // Read channel 1 (bits 0-15)
        let mut ch1_value: u16 = 0;
        for i in 0..16 {
            if *input
                .get(base_bit + i)
                .ok_or_else(|| anyhow::anyhow!("Channel 1 bit {} out of bounds", i))?
            {
                ch1_value |= 1 << i;
            }
        }
        self.txpdo.channel1 = ch1_value as i16;

        // Read channel 2 (bits 16-31)
        let mut ch2_value: u16 = 0;
        for i in 0..16 {
            if *input
                .get(base_bit + 16 + i)
                .ok_or_else(|| anyhow::anyhow!("Channel 2 bit {} out of bounds", i))?
            {
                ch2_value |= 1 << i;
            }
        }
        self.txpdo.channel2 = ch2_value as i16;

        // Read channel 3 (bits 32-47)
        let mut ch3_value: u16 = 0;
        for i in 0..16 {
            if *input
                .get(base_bit + 32 + i)
                .ok_or_else(|| anyhow::anyhow!("Channel 3 bit {} out of bounds", i))?
            {
                ch3_value |= 1 << i;
            }
        }
        self.txpdo.channel3 = ch3_value as i16;

        // Read channel 4 (bits 48-63)
        let mut ch4_value: u16 = 0;
        for i in 0..16 {
            if *input
                .get(base_bit + 48 + i)
                .ok_or_else(|| anyhow::anyhow!("Channel 4 bit {} out of bounds", i))?
            {
                ch4_value |= 1 << i;
            }
        }
        self.txpdo.channel4 = ch4_value as i16;

        Ok(())
    }

    fn input_len(&self) -> usize {
        // 4 channels × 16 bits = 64 bits
        64
    }

    fn output(
        &self,
        _output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        // No outputs for an analog input module
        Ok(())
    }

    fn output_len(&self) -> usize {
        // No outputs
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

impl EthercatDeviceProcessing for Wago750_455 {}

impl NewEthercatDevice for Wago750_455 {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            txpdo: Wago750_455TxPdo::default(),
        }
    }
}

impl std::fmt::Debug for Wago750_455 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_455")
    }
}

// Wago vendor ID and product ID for 750-455
pub const WAGO_750_455_VENDOR_ID: u32 = 0x00000021; // Standard Wago vendor ID
pub const WAGO_750_455_PRODUCT_ID: u32 = 0x45541b3; // Product code for 750-455
pub const WAGO_750_455_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_455_VENDOR_ID, WAGO_750_455_PRODUCT_ID);
