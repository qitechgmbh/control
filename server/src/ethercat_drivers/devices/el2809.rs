use std::any::Any;

use crate::ethercat_drivers::{
    device::Device,
    io::digital_output::{DigitalOutputDevice, DigitalOutputState},
};

const OUTPUT_PDU_LEN: usize = 2;

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
    Pin1,
    Pin2,
    Pin3,
    Pin4,
    Pin5,
    Pin6,
    Pin7,
    Pin8,
    Pin9,
    Pin10,
    Pin11,
    Pin12,
    Pin13,
    Pin14,
    Pin15,
    Pin16,
}

impl EL2809Port {
    pub fn to_pdu_bit_index(&self) -> (usize, usize) {
        match self {
            EL2809Port::Pin1 => (0, 0),
            EL2809Port::Pin2 => (0, 1),
            EL2809Port::Pin3 => (0, 2),
            EL2809Port::Pin4 => (0, 3),
            EL2809Port::Pin5 => (0, 4),
            EL2809Port::Pin6 => (0, 5),
            EL2809Port::Pin7 => (0, 6),
            EL2809Port::Pin8 => (0, 7),
            EL2809Port::Pin9 => (1, 0),
            EL2809Port::Pin10 => (1, 1),
            EL2809Port::Pin11 => (1, 2),
            EL2809Port::Pin12 => (1, 3),
            EL2809Port::Pin13 => (1, 4),
            EL2809Port::Pin14 => (1, 5),
            EL2809Port::Pin15 => (1, 6),
            EL2809Port::Pin16 => (1, 7),
        }
    }
}
