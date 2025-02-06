use crate::{
    device::EthercatDevice,
    io::temperature_input::{
        TemperatureInputDevice, TemperatureInputLimit, TemperatureInputState, TemperatureInputValid,
    },
};
use std::any::Any;

const INPUT_PDU_LEN: usize = 16;

/// EL3204 4-channel temperature input device
///
/// PT100 / Ni100 (RTD) / (2 wire)
#[derive(Debug)]
pub struct EL3204 {
    input_pdu: [u8; INPUT_PDU_LEN],
    pub inputs_ts: u64,
}

impl EL3204 {
    pub fn new() -> Self {
        Self {
            input_pdu: [0; INPUT_PDU_LEN],
            inputs_ts: 0,
        }
    }
}

impl TemperatureInputDevice<EL3204Port> for EL3204 {
    fn temperature_input_state(&self, port: EL3204Port) -> TemperatureInputState {
        let byte_offset = port.to_byte_offset();
        let value = i16::from_be_bytes([
            self.input_pdu[byte_offset + 3],
            self.input_pdu[byte_offset + 2],
        ]);
        let byte_0 = self.input_pdu[byte_offset + 0];
        let byte_1 = self.input_pdu[byte_offset + 1];

        // subindex 01
        let status_undervoltage = byte_0 & 0b0000_0001 != 0;
        //subindex 02
        let status_overvoltage = byte_0 & 0b0000_0010 != 0;
        // subindex 03/04
        let limit_1 = TemperatureInputLimit::new((byte_0 & 0b0000_1100) >> 2 as u8);
        // subindex 05/06
        let limit_2: TemperatureInputLimit =
            TemperatureInputLimit::new((byte_0 & 0b0011_0000) >> 4 as u8);
        // subindex 07
        let error = byte_0 & 0b1000_0000 != 0;
        // subindex 0F 0b0100_0000
        let valid = TemperatureInputValid::new(byte_1 & 0b0100_0000 >> 6);
        // subindex 10 0b1000_0000
        let toggle = byte_1 & 0b1000_0000 != 0;
        let temperature = (value as f32) / 10.0;

        TemperatureInputState {
            input_ts: self.inputs_ts,
            value: temperature,
            status_undervoltage,
            status_overvoltage,
            limit_1,
            limit_2,
            error,
            valid,
            toggle,
        }
    }
}

impl EthercatDevice for EL3204 {
    fn input(&mut self, input: &[u8]) {
        self.input_pdu.copy_from_slice(input);
    }
    fn input_len(&self) -> usize {
        INPUT_PDU_LEN
    }
    fn ts(&mut self, _input_ts: u64, output_ts: u64) {
        self.inputs_ts = output_ts;
    }
    fn as_any(&self) -> &dyn Any {
        self
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
