use super::Device;
use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputState};
use std::any::Any;

const OUTPUT_PDU_LEN: usize = 2;

/// EL2809 16-channel digital output device
/// 24V DC, 0.5A per channel
#[derive(Debug)]
pub struct EL2809 {
    output_pdus: [u8; OUTPUT_PDU_LEN],
    pub output_ts: u64,
}

impl EL2809 {
    pub fn new() -> Self {
        Self {
            output_pdus: [0; OUTPUT_PDU_LEN],
            output_ts: 0,
        }
    }
}

impl DigitalOutputDevice<EL2809Port> for EL2809 {
    fn digital_output_write(&mut self, port: EL2809Port, value: bool) {
        let pdu = match value {
            true => 0b1_u8,
            false => 0b0_u8,
        };
        let (pdu_index, bit_index) = port.to_pdu_bit_index();
        self.output_pdus[pdu_index] =
            (self.output_pdus[pdu_index] & !(1 << bit_index)) | (pdu << bit_index);
    }

    fn digital_output_state(&self, port: EL2809Port) -> DigitalOutputState {
        let (pdu_index, bit_index) = port.to_pdu_bit_index();
        DigitalOutputState {
            output_ts: self.output_ts,
            value: self.output_pdus[pdu_index] & (1 << bit_index) != 0,
        }
    }
}

impl Device for EL2809 {
    fn output_len(&self) -> usize {
        OUTPUT_PDU_LEN
    }
    fn output(&self, output: &mut [u8]) {
        output.copy_from_slice(&self.output_pdus);
    }
    fn ts(&mut self, _input_ts: u64, output_ts: u64) {
        self.output_ts = output_ts;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL2809Port {
    DO1,
    DO2,
    DO3,
    DO4,
    DO5,
    DO6,
    DO7,
    DO8,
    DO9,
    DO10,
    DO11,
    DO12,
    DO13,
    DO14,
    DO15,
    DO16,
}

impl EL2809Port {
    pub fn to_pdu_bit_index(&self) -> (usize, usize) {
        match self {
            EL2809Port::DO1 => (0, 0),
            EL2809Port::DO2 => (0, 1),
            EL2809Port::DO3 => (0, 2),
            EL2809Port::DO4 => (0, 3),
            EL2809Port::DO5 => (0, 4),
            EL2809Port::DO6 => (0, 5),
            EL2809Port::DO7 => (0, 6),
            EL2809Port::DO8 => (0, 7),
            EL2809Port::DO9 => (1, 0),
            EL2809Port::DO10 => (1, 1),
            EL2809Port::DO11 => (1, 2),
            EL2809Port::DO12 => (1, 3),
            EL2809Port::DO13 => (1, 4),
            EL2809Port::DO14 => (1, 5),
            EL2809Port::DO15 => (1, 6),
            EL2809Port::DO16 => (1, 7),
        }
    }
}
