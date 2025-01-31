use crate::ethercat_drivers::{
    device::Device,
    io::digital_input::{DigitalInputDevice, DigitalInputState},
};
use std::any::Any;

const INPUT_PDU_LEN: usize = 1;

/// EL1008 8-channel digital input device
/// 
/// 24V DC, 3ms filter
#[derive(Debug)]
pub struct EL1008 {
    input_pdu: [u8; INPUT_PDU_LEN],
    pub inputs_ts: u64,
}

impl EL1008 {
    pub fn new() -> Self {
        Self {
            input_pdu: [0; INPUT_PDU_LEN],
            inputs_ts: 0,
        }
    }
}

impl DigitalInputDevice<EL1008Port> for EL1008 {
    fn digital_input_state(&self, port: EL1008Port) -> DigitalInputState {
        let bit_index = port.to_bit_index();
        DigitalInputState {
            input_ts: self.inputs_ts,
            value: self.input_pdu[0] & (1 << bit_index) != 0,
        }
    }
}

impl Device for EL1008 {
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
pub enum EL1008Port {
    DI1,
    DI2,
    DI3,
    DI4,
    DI5,
    DI6,
    DI7,
    DI8,
}

impl EL1008Port {
    pub fn to_bit_index(&self) -> usize {
        match self {
            EL1008Port::DI1 => 0,
            EL1008Port::DI2 => 1,
            EL1008Port::DI3 => 2,
            EL1008Port::DI4 => 3,
            EL1008Port::DI5 => 4,
            EL1008Port::DI6 => 5,
            EL1008Port::DI7 => 6,
            EL1008Port::DI8 => 7,
        }
    }
}
