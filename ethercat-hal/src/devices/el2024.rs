use ethercat_hal_derive::{Device, RxPdo, TxPdo};

use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput, DigitalOutputState};
use crate::pdo::basic::BoolPdoObject;
use crate::types::EthercrabSubDevice;

/// EL2024 4-channel digital output device
///
/// 24V DC, 0.5A per channel
#[derive(Debug, Device)]
pub struct EL2024 {
    pub output_ts: u64,
    rxpdu: EL2024RxPdu,
}

impl EL2024 {
    pub fn new() -> Self {
        Self {
            output_ts: 0,
            rxpdu: EL2024RxPdu::default(),
        }
    }
}

impl DigitalOutputDevice<EL2024Port> for EL2024 {
    fn digital_output_write(&mut self, _port: EL2024Port, value: bool) {
        let _pdu = match value {
            true => 0b1,
            false => 0b0,
        };
        todo!();
        // let bit_index = port.to_bit_index();
        // self.output_pdus[0] = (self.output_pdus[0] & !(1 << bit_index)) | (pdu << bit_index);
    }

    fn digital_output_state(&self, port: EL2024Port) -> DigitalOutputState {
        DigitalOutputState {
            output_ts: self.output_ts,
            output: DigitalOutputOutput {
                value: match port {
                    EL2024Port::DO1 => self.rxpdu.channel1.as_ref().unwrap().value,
                    EL2024Port::DO2 => self.rxpdu.channel2.as_ref().unwrap().value,
                    EL2024Port::DO3 => self.rxpdu.channel3.as_ref().unwrap().value,
                    EL2024Port::DO4 => self.rxpdu.channel4.as_ref().unwrap().value,
                },
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum EL2024Port {
    DO1,
    DO2,
    DO3,
    DO4,
}

#[derive(Debug, Clone, RxPdo, Default)]
struct EL2024RxPdu {
    #[pdo_object_index(0x1600)]
    pub channel1: Option<BoolPdoObject>,
    #[pdo_object_index(0x1601)]
    pub channel2: Option<BoolPdoObject>,
    #[pdo_object_index(0x1602)]
    pub channel3: Option<BoolPdoObject>,
    #[pdo_object_index(0x1603)]
    pub channel4: Option<BoolPdoObject>,
}

#[derive(Debug, Clone, TxPdo, Default)]
pub struct EL2024TxPdu {}
