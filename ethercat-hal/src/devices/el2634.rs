use ethercat_hal_derive::{Device, RxPdo, TxPdo};

use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput, DigitalOutputState};
use crate::pdo::basic::BoolPdoObject;
use crate::types::EthercrabSubDevice;

/// EL2634 4-channel relay device
///
/// 250V AC / 30V DC / 4A per channel
#[derive(Debug, Device)]
pub struct EL2634 {
    pub output_ts: u64,
    rxpdu: EL2634RxPdu,
}

impl EL2634 {
    pub fn new() -> Self {
        Self {
            output_ts: 0,
            rxpdu: EL2634RxPdu::default(),
        }
    }
}

impl DigitalOutputDevice<EL2634Port> for EL2634 {
    fn digital_output_write(&mut self, _port: EL2634Port, value: bool) {
        let _pdu = match value {
            true => 0b1,
            false => 0b0,
        };
        todo!();
        // let bit_index = port.to_bit_index();
        // self.output_pdus[0] = (self.output_pdus[0] & !(1 << bit_index)) | (pdu << bit_index);
    }

    fn digital_output_state(&self, port: EL2634Port) -> DigitalOutputState {
        DigitalOutputState {
            output_ts: self.output_ts,
            output: DigitalOutputOutput {
                value: match port {
                    EL2634Port::R1 => self.rxpdu.channel1.as_ref().unwrap().value,
                    EL2634Port::R2 => self.rxpdu.channel2.as_ref().unwrap().value,
                    EL2634Port::R3 => self.rxpdu.channel3.as_ref().unwrap().value,
                    EL2634Port::R4 => self.rxpdu.channel4.as_ref().unwrap().value,
                },
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum EL2634Port {
    R1,
    R2,
    R3,
    R4,
}

#[derive(Debug, Clone, RxPdo, Default)]
struct EL2634RxPdu {
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
pub struct EL2634TxPdu {}
