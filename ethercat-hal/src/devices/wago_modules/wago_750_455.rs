use units::{ElectricCurrent, electric_current::milliampere};

use crate::{
    devices::{
        DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
        EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
    },
    helpers::signing_converter_u16::U16SigningConverter,
    io::{
        analog_input::{AnalogInputDevice, AnalogInputInput, physical::AnalogInputRange},
        digital_input::DigitalInputInput,
    },
};

Wago750_455Presentation {

}

#[derive(Clone)]
pub struct Wago750_455 {
    configuration: (),
    txpdo: Wago750_455RxPdo,
    isUsed: bool,
}

pub struct Wago750_455Configuration {

}

#[derive(Debug, Clone)]
pub enum Wago750_455InputPort {
    AI1,
    AI2,
    AI3,
    AI4,
}

impl From<Wago750_455InputPort> for usize {
    fn from(value: Wago750_455InputPort) -> Self {
        match value {
            Wago750_455InputPort::AI1 => 0,
            Wago750_455InputPort::AI2 => 1,
            Wago750_455InputPort::AI3 => 2,
            Wago750_455InputPort::AI4 => 3,
        }
    }
}

#[derive(Clone, Default)]
/**
# Data Word, 16 bits
- B15 ... B4 hold the measured value (B15 is the sign bit which should always be 0)
- B3 and B2 are 00 when measuring <~3.0mA, 11 when measuring >~21.0mA and undefined inbetween.
- B1 and B0 are 11' on an overflow, underflow or wire break. 00' otherwise
*/
pub struct Wago750_455RxPdo {
    /** Low Byte: D0, High Byte: D1 */
    channel_1: u16,
    /** Low Byte: D2, High Byte: D3 */
    channel_2: u16,
    /** Low Byte: D4, High Byte: D5 */
    channel_3: u16,
    /** Low Byte: D6, High Byte: D7 */
    channel_4: u16,
}

impl AnalogInputDevice<Wago750_455InputPort> for Wago750_455 {
    fn get_input(&self, port: Wago750_455InputPort) -> AnalogInputInput {
        let raw_value = match port {
            Wago750_455InputPort::AI1 => self.txpdo.channel_1,
            Wago750_455InputPort::AI2 => self.txpdo.channel_2,
            Wago750_455InputPort::AI3 => self.txpdo.channel_3,
            Wago750_455InputPort::AI4 => self.txpdo.channel_4,
        };

        let signing_bit = raw_value & 0x8000;
        // The 4 least significant bits are reserved and diagnostic and the MSB is the signing bit.
        let shifted_raw_value = (raw_value & 0x7FF0) >> 4;
        // Since only 11 bits actually represent the measurement (ignoring the signing bit) only 2048 different states can be measured.
        let prepared_value = U16SigningConverter::load_raw(shifted_raw_value | signing_bit);

        let value = prepared_value.as_signed() as i16;

        let normalized = f32::from(value) / f32::from(2047);

        AnalogInputInput {
            normalized,
            wiring_error: false,
        }
    }

    fn analog_input_range(&self) -> AnalogInputRange {
        AnalogInputRange::Current {
            min: ElectricCurrent::new::<milliampere>(4.0),
            max: ElectricCurrent::new::<milliampere>(20.0),
            min_raw: 0,
            max_raw: 2047,
        }
    }
}
