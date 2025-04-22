use super::Device;
use crate::io::analog_output::{AnalogOutputDevice, AnalogOutputState};
use std::any::Any;

const OUTPUT_PDU_LEN: usize = 16;

/// EL4008 8-channel analog output device
///
/// 12-bit resolution, 0-10V
///
/// load > 5kOhm
pub struct EL4008 {
    output_pdus: [u8; OUTPUT_PDU_LEN],
}

impl std::fmt::Debug for EL4008 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EL4008")
    }
}

impl EL4008 {
    pub fn new() -> Self {
        Self {
            output_pdus: [0; OUTPUT_PDU_LEN],
        }
    }
}

/// transform a float from 0-1 to a 12bit integer
fn f32_to_i16(value: f32) -> i16 {
    // Clamp the value between -1.0 and 1.0
    let clamped_value = value.clamp(-1.0, 1.0);
    // Scale to the 16-bit range and round
    let scaled_value = (clamped_value * 32767.0).round();
    // Cast to i16
    scaled_value as i16
}

impl AnalogOutputDevice<EL4008Port> for EL4008 {
    fn analog_output_write(&mut self, port: EL4008Port, value: f32) {
        let pdu_index = port.to_le_pdu_index();
        let value = f32_to_i16(value);
        let bytes = value.to_be_bytes();
        self.output_pdus[pdu_index + 1] = bytes[0];
        self.output_pdus[pdu_index + 0] = bytes[1];
    }

    fn analog_output_state(&self, port: EL4008Port) -> AnalogOutputState {
        let pdu_index = port.to_le_pdu_index();
        // turn 2 bytes into a single f32
        let value = f32::from_le_bytes([
            0,
            0,
            self.output_pdus[pdu_index + 1],
            self.output_pdus[pdu_index],
        ]);
        AnalogOutputState { value }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL4008Port {
    AO1,
    AO2,
    AO3,
    AO4,
    AO5,
    AO6,
    AO7,
    AO8,
}

impl EL4008Port {
    pub fn to_le_pdu_index(&self) -> usize {
        match self {
            EL4008Port::AO1 => 0,
            EL4008Port::AO2 => 2,
            EL4008Port::AO3 => 4,
            EL4008Port::AO4 => 6,
            EL4008Port::AO5 => 8,
            EL4008Port::AO6 => 10,
            EL4008Port::AO7 => 12,
            EL4008Port::AO8 => 14,
        }
    }
}
