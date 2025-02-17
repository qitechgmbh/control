use ethercat_hal_derive::{Device, RxPdo, TxPdo};

use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput, DigitalOutputState};
use crate::pdo::basic::BoolPdoObject;
use crate::types::EthercrabSubDevice;

/// EL2809 16-channel digital output device
///
/// 24V DC, 0.5A per channel
#[derive(Debug, Device)]
pub struct EL2809 {
    pub output_ts: u64,
    rxpdu: EL2809RxPdu,
}

impl EL2809 {
    pub fn new() -> Self {
        Self {
            output_ts: 0,
            rxpdu: EL2809RxPdu::default(),
        }
    }
}

impl DigitalOutputDevice<EL2809Port> for EL2809 {
    fn digital_output_write(&mut self, _port: EL2809Port, value: bool) {
        let _pdu = match value {
            true => 0b1,
            false => 0b0,
        };
        todo!();
        // let bit_index = port.to_bit_index();
        // self.output_pdus[0] = (self.output_pdus[0] & !(1 << bit_index)) | (pdu << bit_index);
    }

    fn digital_output_state(&self, port: EL2809Port) -> DigitalOutputState {
        DigitalOutputState {
            output_ts: self.output_ts,
            output: DigitalOutputOutput {
                value: match port {
                    EL2809Port::DO1 => self.rxpdu.channel1.as_ref().unwrap().value,
                    EL2809Port::DO2 => self.rxpdu.channel2.as_ref().unwrap().value,
                    EL2809Port::DO3 => self.rxpdu.channel3.as_ref().unwrap().value,
                    EL2809Port::DO4 => self.rxpdu.channel4.as_ref().unwrap().value,
                    EL2809Port::DO5 => self.rxpdu.channel5.as_ref().unwrap().value,
                    EL2809Port::DO6 => self.rxpdu.channel6.as_ref().unwrap().value,
                    EL2809Port::DO7 => self.rxpdu.channel7.as_ref().unwrap().value,
                    EL2809Port::DO8 => self.rxpdu.channel8.as_ref().unwrap().value,
                    EL2809Port::DO9 => self.rxpdu.channel9.as_ref().unwrap().value,
                    EL2809Port::DO10 => self.rxpdu.channel10.as_ref().unwrap().value,
                    EL2809Port::DO11 => self.rxpdu.channel11.as_ref().unwrap().value,
                    EL2809Port::DO12 => self.rxpdu.channel12.as_ref().unwrap().value,
                    EL2809Port::DO13 => self.rxpdu.channel13.as_ref().unwrap().value,
                    EL2809Port::DO14 => self.rxpdu.channel14.as_ref().unwrap().value,
                    EL2809Port::DO15 => self.rxpdu.channel15.as_ref().unwrap().value,
                    EL2809Port::DO16 => self.rxpdu.channel16.as_ref().unwrap().value,
                },
            },
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, RxPdo, Default)]
struct EL2809RxPdu {
    #[pdo_object_index(0x1600)]
    pub channel1: Option<BoolPdoObject>,
    #[pdo_object_index(0x1601)]
    pub channel2: Option<BoolPdoObject>,
    #[pdo_object_index(0x1602)]
    pub channel3: Option<BoolPdoObject>,
    #[pdo_object_index(0x1603)]
    pub channel4: Option<BoolPdoObject>,
    #[pdo_object_index(0x1604)]
    pub channel5: Option<BoolPdoObject>,
    #[pdo_object_index(0x1605)]
    pub channel6: Option<BoolPdoObject>,
    #[pdo_object_index(0x1606)]
    pub channel7: Option<BoolPdoObject>,
    #[pdo_object_index(0x1607)]
    pub channel8: Option<BoolPdoObject>,
    #[pdo_object_index(0x1608)]
    pub channel9: Option<BoolPdoObject>,
    #[pdo_object_index(0x1609)]
    pub channel10: Option<BoolPdoObject>,
    #[pdo_object_index(0x160A)]
    pub channel11: Option<BoolPdoObject>,
    #[pdo_object_index(0x160B)]
    pub channel12: Option<BoolPdoObject>,
    #[pdo_object_index(0x160C)]
    pub channel13: Option<BoolPdoObject>,
    #[pdo_object_index(0x160D)]
    pub channel14: Option<BoolPdoObject>,
    #[pdo_object_index(0x160E)]
    pub channel15: Option<BoolPdoObject>,
    #[pdo_object_index(0x160F)]
    pub channel16: Option<BoolPdoObject>,
}

#[derive(Debug, Clone, TxPdo, Default)]
pub struct EL2809TxPdu {}
