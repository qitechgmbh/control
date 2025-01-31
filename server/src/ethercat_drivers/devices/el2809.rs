use std::any::Any;

use crate::ethercat_drivers::{
    device::Device,
    io::digital_output::{DigitalOutputDevice, DigitalOutputState},
};

const OUTPUT_PDU_LEN: usize = 16;

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
        let pdu_index = port.to_pdu_index();
        self.output_pdus[pdu_index] = pdu;
    }

    fn digital_output_state(&self, port: EL2809Port) -> DigitalOutputState {
        let bit_index = port.to_pdu_index();
        DigitalOutputState {
            output_ts: self.output_ts,
            value: self.output_pdus[bit_index] != 0,
        }
    }
}

impl Device for EL2809 {
    fn output_len(&self) -> usize {
        OUTPUT_PDU_LEN
    }
    fn output(&self, _output_ts: u64, output: &mut [u8]) {
        output.copy_from_slice(&self.output_pdus);
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
    pub fn to_pdu_index(&self) -> usize {
        match self {
            EL2809Port::Pin1 => 0,
            EL2809Port::Pin2 => 1,
            EL2809Port::Pin3 => 2,
            EL2809Port::Pin4 => 3,
            EL2809Port::Pin5 => 4,
            EL2809Port::Pin6 => 5,
            EL2809Port::Pin7 => 6,
            EL2809Port::Pin8 => 7,
            EL2809Port::Pin9 => 8,
            EL2809Port::Pin10 => 9,
            EL2809Port::Pin11 => 10,
            EL2809Port::Pin12 => 11,
            EL2809Port::Pin13 => 12,
            EL2809Port::Pin14 => 13,
            EL2809Port::Pin15 => 14,
            EL2809Port::Pin16 => 15,
        }
    }
}
