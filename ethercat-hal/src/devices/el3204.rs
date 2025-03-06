use crate::pdo::TxPdo;
use ethercat_hal_derive::{Device, TxPdo};

use crate::types::EthercrabSubDevicePreoperational;
use crate::{
    io::temperature_input::{TemperatureInputDevice, TemperatureInputInput, TemperatureInputState},
    pdo::el32xx::RtdInput,
};
/// EL3204 4-channel temperature input device
///
/// PT100 / Ni100 (RTD) / (2 wire)
#[derive(Debug, Device)]
pub struct EL3204 {
    pub txpdo: EL3204TxPdo,
    pub input_ts: u64,
}

impl EL3204 {
    pub fn new() -> Self {
        Self {
            txpdo: EL3204TxPdo::default(),
            input_ts: 0,
        }
    }
}

impl TemperatureInputDevice<EL3204Port> for EL3204 {
    fn temperature_input_state(&self, port: EL3204Port) -> TemperatureInputState {
        let channel = match port {
            EL3204Port::T1 => self.txpdo.channel1.as_ref().unwrap(),
            EL3204Port::T2 => self.txpdo.channel2.as_ref().unwrap(),
            EL3204Port::T3 => self.txpdo.channel3.as_ref().unwrap(),
            EL3204Port::T4 => self.txpdo.channel4.as_ref().unwrap(),
        };
        TemperatureInputState {
            input_ts: self.input_ts,
            input: TemperatureInputInput {
                temperature: channel.temperature,
                undervoltage: channel.undervoltage,
                overvoltage: channel.overvoltage,
                limit1: channel.limit1,
                limit2: channel.limit2,
                error: channel.error,
                txpdo_state: channel.txpdo_state,
                txpdo_toggle: channel.txpdo_toggle,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL3204Port {
    T1,
    T2,
    T3,
    T4,
}

impl EL3204Port {
    pub fn to_byte_offset(&self) -> usize {
        match self {
            EL3204Port::T1 => 0,
            EL3204Port::T2 => 4,
            EL3204Port::T3 => 8,
            EL3204Port::T4 => 12,
        }
    }
}

#[derive(Debug, Clone, TxPdo, Default)]
pub struct EL3204TxPdo {
    #[pdo_object_index(0x1A00)]
    channel1: Option<RtdInput>,
    #[pdo_object_index(0x1A01)]
    channel2: Option<RtdInput>,
    #[pdo_object_index(0x1A02)]
    channel3: Option<RtdInput>,
    #[pdo_object_index(0x1A03)]
    channel4: Option<RtdInput>,
}
