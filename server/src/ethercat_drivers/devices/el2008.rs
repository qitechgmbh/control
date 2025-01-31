use crate::ethercat_drivers::{
    device::Device,
    io::digital_output::{DigitalOutputDevice, DigitalOutputState},
};
use std::any::Any;

const OUTPUT_PDU_LEN: usize = 1;

#[derive(Debug)]
pub struct EL2008 {
    output_pdus: [u8; OUTPUT_PDU_LEN],
    pub output_ts: u64,
}

impl EL2008 {
    pub fn new() -> Self {
        Self {
            output_pdus: [0; OUTPUT_PDU_LEN],
            output_ts: 0,
        }
    }
}

impl DigitalOutputDevice<EL2008Port> for EL2008 {
    fn digital_output_write(&mut self, port: EL2008Port, value: bool) {
        let pdu = match value {
            true => 0b1,
            false => 0b0,
        };
        let bit_index = port.to_bit_index();
        self.output_pdus[0] = (self.output_pdus[0] & !(1 << bit_index)) | (pdu << bit_index);
    }

    fn digital_output_state(&self, port: EL2008Port) -> DigitalOutputState {
        let bit_index = port.to_bit_index();
        DigitalOutputState {
            output_ts: self.output_ts,
            value: self.output_pdus[0] & (1 << bit_index) != 0,
        }
    }
}

impl Device for EL2008 {
    fn output(&self, output: &mut [u8]) {
        output.copy_from_slice(&self.output_pdus);
    }
    fn output_len(&self) -> usize {
        OUTPUT_PDU_LEN
    }
    fn ts(&mut self, _input_ts: u64, output_ts: u64) {
        self.output_ts = output_ts;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL2008Port {
    Pin1,
    Pin2,
    Pin3,
    Pin4,
    Pin5,
    Pin6,
    Pin7,
    Pin8,
}

impl EL2008Port {
    pub fn to_bit_index(&self) -> usize {
        match self {
            EL2008Port::Pin1 => 0,
            EL2008Port::Pin2 => 1,
            EL2008Port::Pin3 => 2,
            EL2008Port::Pin4 => 3,
            EL2008Port::Pin5 => 4,
            EL2008Port::Pin6 => 5,
            EL2008Port::Pin7 => 6,
            EL2008Port::Pin8 => 7,
        }
    }
}
