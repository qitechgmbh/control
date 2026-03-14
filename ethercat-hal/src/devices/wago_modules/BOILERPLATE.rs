// =============================================================================
// WAGO 750-XXX Device Driver Boilerplate
// =============================================================================
//
// HOW TO USE:
// 1. Copy this file and rename it to `wago_750_XXX.rs`
// 2. Find & replace the following placeholders:
//    - `Wago750_XXX`  -> Your device struct name    (e.g. `Wago750_554`)
//    - `WAGO_750_XXX` -> Your constant prefix       (e.g. `WAGO_750_554`)
//    - `750_XXX`      -> Your module number          (e.g. `750_554`)
// 3. Uncomment one of the device type sections (A through E)
// 4. Delete all other device type sections
// 5. Adjust port count / PDO fields to match your device
// 6. Set PRODUCT_ID at the bottom
// 7. Add `pub mod wago_750_XXX;` to mod.rs
//
// =============================================================================

use crate::devices::{
    DynamicEthercatDevice, EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed,
    EthercatDynamicPDO, Module, NewEthercatDevice, SubDeviceProductTuple,
};

// =============================================================================
// DEVICE TYPE SECTIONS — Uncomment ONE, delete the rest
// =============================================================================

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║ Section A: Digital Input                                                 ║
// ╚═══════════════════════════════════════════════════════════════════════════╝
//
// use crate::io::digital_input::{DigitalInputDevice, DigitalInputInput};
//
// #[derive(Debug, Clone)]
// pub enum Wago750_XXXInputPort {
//     DI1,
//     DI2,
//     DI3,
//     DI4,
// }
//
// impl From<Wago750_XXXInputPort> for usize {
//     fn from(value: Wago750_XXXInputPort) -> Self {
//         match value {
//             Wago750_XXXInputPort::DI1 => 0,
//             Wago750_XXXInputPort::DI2 => 1,
//             Wago750_XXXInputPort::DI3 => 2,
//             Wago750_XXXInputPort::DI4 => 3,
//         }
//     }
// }
//
// #[derive(Clone, Default)]
// pub struct Wago750_XXXTxPdo {
//     port1: bool,
//     port2: bool,
//     port3: bool,
//     port4: bool,
// }
//
// impl DigitalInputDevice<Wago750_XXXInputPort> for Wago750_XXX {
//     fn get_input(&self, port: Wago750_XXXInputPort) -> Result<DigitalInputInput, anyhow::Error> {
//         Ok(DigitalInputInput {
//             value: match port {
//                 Wago750_XXXInputPort::DI1 => self.tx_pdo.port1,
//                 Wago750_XXXInputPort::DI2 => self.tx_pdo.port2,
//                 Wago750_XXXInputPort::DI3 => self.tx_pdo.port3,
//                 Wago750_XXXInputPort::DI4 => self.tx_pdo.port4,
//             },
//         })
//     }
// }
//
// // Paste into EthercatDevice::input():
// //     let base = self.tx_bit_offset;
// //     self.tx_pdo.port1 = *input.get(base + Into::<usize>::into(Wago750_XXXInputPort::DI1)).expect("Bit 1 out of bounds");
// //     self.tx_pdo.port2 = *input.get(base + Into::<usize>::into(Wago750_XXXInputPort::DI2)).expect("Bit 2 out of bounds");
// //     self.tx_pdo.port3 = *input.get(base + Into::<usize>::into(Wago750_XXXInputPort::DI3)).expect("Bit 3 out of bounds");
// //     self.tx_pdo.port4 = *input.get(base + Into::<usize>::into(Wago750_XXXInputPort::DI4)).expect("Bit 4 out of bounds");
// //
// // input_len: 4  (number of digital input ports)
// // output_len: 0
// // output(): Ok(())
// // Struct fields: tx_pdo: Wago750_XXXTxPdo
// // NewEthercatDevice fields: tx_pdo: Wago750_XXXTxPdo::default()

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║ Section B: Digital Output                                                ║
// ╚═══════════════════════════════════════════════════════════════════════════╝
//
// use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};
//
// #[derive(Debug, Clone)]
// pub enum Wago750_XXXOutputPort {
//     DO1,
//     DO2,
//     DO3,
//     DO4,
// }
//
// impl From<Wago750_XXXOutputPort> for usize {
//     fn from(value: Wago750_XXXOutputPort) -> Self {
//         match value {
//             Wago750_XXXOutputPort::DO1 => 0,
//             Wago750_XXXOutputPort::DO2 => 1,
//             Wago750_XXXOutputPort::DO3 => 2,
//             Wago750_XXXOutputPort::DO4 => 3,
//         }
//     }
// }
//
// #[derive(Clone, Default)]
// pub struct Wago750_XXXRxPdo {
//     port1: bool,
//     port2: bool,
//     port3: bool,
//     port4: bool,
// }
//
// impl DigitalOutputDevice<Wago750_XXXOutputPort> for Wago750_XXX {
//     fn set_output(&mut self, port: Wago750_XXXOutputPort, value: DigitalOutputOutput) {
//         let output_value: bool = value.into();
//         match port {
//             Wago750_XXXOutputPort::DO1 => self.rx_pdo.port1 = output_value,
//             Wago750_XXXOutputPort::DO2 => self.rx_pdo.port2 = output_value,
//             Wago750_XXXOutputPort::DO3 => self.rx_pdo.port3 = output_value,
//             Wago750_XXXOutputPort::DO4 => self.rx_pdo.port4 = output_value,
//         }
//     }
//
//     fn get_output(&self, port: Wago750_XXXOutputPort) -> DigitalOutputOutput {
//         let current_value = match port {
//             Wago750_XXXOutputPort::DO1 => self.rx_pdo.port1,
//             Wago750_XXXOutputPort::DO2 => self.rx_pdo.port2,
//             Wago750_XXXOutputPort::DO3 => self.rx_pdo.port3,
//             Wago750_XXXOutputPort::DO4 => self.rx_pdo.port4,
//         };
//         DigitalOutputOutput(current_value)
//     }
// }
//
// // Paste into EthercatDevice::output():
// //     output.set(self.rx_bit_offset + Into::<usize>::into(Wago750_XXXOutputPort::DO1), self.rx_pdo.port1);
// //     output.set(self.rx_bit_offset + Into::<usize>::into(Wago750_XXXOutputPort::DO2), self.rx_pdo.port2);
// //     output.set(self.rx_bit_offset + Into::<usize>::into(Wago750_XXXOutputPort::DO3), self.rx_pdo.port3);
// //     output.set(self.rx_bit_offset + Into::<usize>::into(Wago750_XXXOutputPort::DO4), self.rx_pdo.port4);
// //
// // input_len: 0
// // output_len: 4  (number of digital output ports)
// // input(): Ok(())
// // Struct fields: rx_pdo: Wago750_XXXRxPdo
// // NewEthercatDevice fields: rx_pdo: Wago750_XXXRxPdo::default()

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║ Section C: Digital Input + Output (combined)                             ║
// ╚═══════════════════════════════════════════════════════════════════════════╝
//
// use crate::io::digital_input::{DigitalInputDevice, DigitalInputInput};
// use crate::io::digital_output::{DigitalOutputDevice, DigitalOutputOutput};
//
// #[derive(Debug, Clone)]
// pub enum Wago750_XXXInputPort {
//     DI1,
//     DI2,
//     DI3,
//     DI4,
// }
//
// impl From<Wago750_XXXInputPort> for usize {
//     fn from(value: Wago750_XXXInputPort) -> Self {
//         match value {
//             Wago750_XXXInputPort::DI1 => 0,
//             Wago750_XXXInputPort::DI2 => 1,
//             Wago750_XXXInputPort::DI3 => 2,
//             Wago750_XXXInputPort::DI4 => 3,
//         }
//     }
// }
//
// #[derive(Debug, Clone)]
// pub enum Wago750_XXXOutputPort {
//     DO1,
//     DO2,
//     DO3,
//     DO4,
// }
//
// impl From<Wago750_XXXOutputPort> for usize {
//     fn from(value: Wago750_XXXOutputPort) -> Self {
//         match value {
//             Wago750_XXXOutputPort::DO1 => 0,
//             Wago750_XXXOutputPort::DO2 => 1,
//             Wago750_XXXOutputPort::DO3 => 2,
//             Wago750_XXXOutputPort::DO4 => 3,
//         }
//     }
// }
//
// #[derive(Clone, Default)]
// pub struct Wago750_XXXTxPdo {
//     port1: bool,
//     port2: bool,
//     port3: bool,
//     port4: bool,
// }
//
// #[derive(Clone, Default)]
// pub struct Wago750_XXXRxPdo {
//     port1: bool,
//     port2: bool,
//     port3: bool,
//     port4: bool,
// }
//
// impl DigitalInputDevice<Wago750_XXXInputPort> for Wago750_XXX {
//     fn get_input(&self, port: Wago750_XXXInputPort) -> Result<DigitalInputInput, anyhow::Error> {
//         Ok(DigitalInputInput {
//             value: match port {
//                 Wago750_XXXInputPort::DI1 => self.tx_pdo.port1,
//                 Wago750_XXXInputPort::DI2 => self.tx_pdo.port2,
//                 Wago750_XXXInputPort::DI3 => self.tx_pdo.port3,
//                 Wago750_XXXInputPort::DI4 => self.tx_pdo.port4,
//             },
//         })
//     }
// }
//
// impl DigitalOutputDevice<Wago750_XXXOutputPort> for Wago750_XXX {
//     fn set_output(&mut self, port: Wago750_XXXOutputPort, value: DigitalOutputOutput) {
//         let output_value: bool = value.into();
//         match port {
//             Wago750_XXXOutputPort::DO1 => self.rx_pdo.port1 = output_value,
//             Wago750_XXXOutputPort::DO2 => self.rx_pdo.port2 = output_value,
//             Wago750_XXXOutputPort::DO3 => self.rx_pdo.port3 = output_value,
//             Wago750_XXXOutputPort::DO4 => self.rx_pdo.port4 = output_value,
//         }
//     }
//
//     fn get_output(&self, port: Wago750_XXXOutputPort) -> DigitalOutputOutput {
//         let current_value = match port {
//             Wago750_XXXOutputPort::DO1 => self.rx_pdo.port1,
//             Wago750_XXXOutputPort::DO2 => self.rx_pdo.port2,
//             Wago750_XXXOutputPort::DO3 => self.rx_pdo.port3,
//             Wago750_XXXOutputPort::DO4 => self.rx_pdo.port4,
//         };
//         DigitalOutputOutput(current_value)
//     }
// }
//
// // Paste into EthercatDevice::input():
// //     let base = self.tx_bit_offset;
// //     self.tx_pdo.port1 = *input.get(base + Into::<usize>::into(Wago750_XXXInputPort::DI1)).expect("Bit 1 out of bounds");
// //     self.tx_pdo.port2 = *input.get(base + Into::<usize>::into(Wago750_XXXInputPort::DI2)).expect("Bit 2 out of bounds");
// //     self.tx_pdo.port3 = *input.get(base + Into::<usize>::into(Wago750_XXXInputPort::DI3)).expect("Bit 3 out of bounds");
// //     self.tx_pdo.port4 = *input.get(base + Into::<usize>::into(Wago750_XXXInputPort::DI4)).expect("Bit 4 out of bounds");
// //
// // Paste into EthercatDevice::output():
// //     output.set(self.rx_bit_offset + Into::<usize>::into(Wago750_XXXOutputPort::DO1), self.rx_pdo.port1);
// //     output.set(self.rx_bit_offset + Into::<usize>::into(Wago750_XXXOutputPort::DO2), self.rx_pdo.port2);
// //     output.set(self.rx_bit_offset + Into::<usize>::into(Wago750_XXXOutputPort::DO3), self.rx_pdo.port3);
// //     output.set(self.rx_bit_offset + Into::<usize>::into(Wago750_XXXOutputPort::DO4), self.rx_pdo.port4);
// //
// // input_len: 4  (number of digital input ports)
// // output_len: 4  (number of digital output ports)
// // Struct fields: tx_pdo: Wago750_XXXTxPdo, rx_pdo: Wago750_XXXRxPdo
// // NewEthercatDevice fields: tx_pdo: Wago750_XXXTxPdo::default(), rx_pdo: Wago750_XXXRxPdo::default()

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║ Section D: Analog Input                                                  ║
// ╚═══════════════════════════════════════════════════════════════════════════╝
//
// use bitvec::field::BitField;
// use crate::io::analog_input::physical::AnalogInputRange;
// use crate::io::analog_input::{AnalogInputDevice, AnalogInputInput};
//
// #[derive(Debug, Clone)]
// pub enum Wago750_XXXPort {
//     AI1,
//     AI2,
//     AI3,
//     AI4,
// }
//
// impl From<Wago750_XXXPort> for usize {
//     fn from(value: Wago750_XXXPort) -> Self {
//         match value {
//             Wago750_XXXPort::AI1 => 0,
//             Wago750_XXXPort::AI2 => 16,
//             Wago750_XXXPort::AI3 => 32,
//             Wago750_XXXPort::AI4 => 48,
//         }
//     }
// }
//
// #[derive(Clone, Default)]
// pub struct Wago750_XXXTxPdo {
//     ai1: u16,
//     ai2: u16,
//     ai3: u16,
//     ai4: u16,
// }
//
// impl AnalogInputDevice<Wago750_XXXPort> for Wago750_XXX {
//     fn get_input(&self, port: Wago750_XXXPort) -> AnalogInputInput {
//         let raw = match port {
//             Wago750_XXXPort::AI1 => self.tx_pdo.ai1,
//             Wago750_XXXPort::AI2 => self.tx_pdo.ai2,
//             Wago750_XXXPort::AI3 => self.tx_pdo.ai3,
//             Wago750_XXXPort::AI4 => self.tx_pdo.ai4,
//         };
//         let wiring_error = (raw & 0x0003) == 0x0003;
//         let raw_value = (raw & 0x7FF0) as i16;
//         let normalized = self.analog_input_range().raw_to_normalized(raw_value) as f32;
//         AnalogInputInput {
//             normalized,
//             wiring_error,
//         }
//     }
//
//     fn analog_input_range(&self) -> AnalogInputRange {
//         // Adjust to match your device. Common examples:
//         //   Current 4-20mA:  AnalogInputRange::Current { min: ..milliampere(4.0), max: ..milliampere(20.0), min_raw: 0, max_raw: 0x7FF0 }
//         //   Voltage 0-10V:   AnalogInputRange::Voltage { min: ..volt(0.0), max: ..volt(10.0), min_raw: 0, max_raw: 0x7FF0 }
//         todo!("Set the analog input range for your device")
//     }
// }
//
// // Paste into EthercatDevice::input():
// //     let base = self.tx_bit_offset;
// //     self.tx_pdo.ai1 = input[base..(base + 16)].load_le::<u16>();
// //     self.tx_pdo.ai2 = input[(base + 16)..(base + 32)].load_le::<u16>();
// //     self.tx_pdo.ai3 = input[(base + 32)..(base + 48)].load_le::<u16>();
// //     self.tx_pdo.ai4 = input[(base + 48)..(base + 64)].load_le::<u16>();
// //
// // input_len: 64  (channels * 16 bits)
// // output_len: 0
// // output(): Ok(())
// // input_checked(): self.input(_input)   (delegate, not Ok(()))
// // Struct fields: tx_pdo: Wago750_XXXTxPdo
// // NewEthercatDevice fields: tx_pdo: Wago750_XXXTxPdo::default()

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║ Section E: Analog Output                                                 ║
// ╚═══════════════════════════════════════════════════════════════════════════╝
// AnalogOutputOutput is a newtype around f32 in clip space (0.0..1.0).
// Convert to/from raw u16 using your device's raw range (e.g. 0x0000..0x7FFF).
//
// use bitvec::field::BitField;
// use crate::io::analog_output::{AnalogOutputDevice, AnalogOutputOutput};
//
// #[derive(Debug, Clone)]
// pub enum Wago750_XXXPort {
//     AO1,
//     AO2,
//     AO3,
//     AO4,
// }
//
// impl From<Wago750_XXXPort> for usize {
//     fn from(value: Wago750_XXXPort) -> Self {
//         match value {
//             Wago750_XXXPort::AO1 => 0,
//             Wago750_XXXPort::AO2 => 16,
//             Wago750_XXXPort::AO3 => 32,
//             Wago750_XXXPort::AO4 => 48,
//         }
//     }
// }
//
// #[derive(Clone, Default)]
// pub struct Wago750_XXXRxPdo {
//     ao1: u16,
//     ao2: u16,
//     ao3: u16,
//     ao4: u16,
// }
//
// impl AnalogOutputDevice<Wago750_XXXPort> for Wago750_XXX {
//     fn set_output(&mut self, port: Wago750_XXXPort, value: AnalogOutputOutput) {
//         // Convert clip-space f32 (0.0..1.0) to raw u16 — adjust 0x7FFF to your device's max
//         let raw = (value.0.clamp(0.0, 1.0) * 0x7FFF as f32) as u16;
//         match port {
//             Wago750_XXXPort::AO1 => self.rx_pdo.ao1 = raw,
//             Wago750_XXXPort::AO2 => self.rx_pdo.ao2 = raw,
//             Wago750_XXXPort::AO3 => self.rx_pdo.ao3 = raw,
//             Wago750_XXXPort::AO4 => self.rx_pdo.ao4 = raw,
//         }
//     }
//
//     fn get_output(&self, port: Wago750_XXXPort) -> AnalogOutputOutput {
//         let raw = match port {
//             Wago750_XXXPort::AO1 => self.rx_pdo.ao1,
//             Wago750_XXXPort::AO2 => self.rx_pdo.ao2,
//             Wago750_XXXPort::AO3 => self.rx_pdo.ao3,
//             Wago750_XXXPort::AO4 => self.rx_pdo.ao4,
//         };
//         // Convert raw u16 back to clip-space f32 — adjust 0x7FFF to your device's max
//         AnalogOutputOutput(raw as f32 / 0x7FFF as f32)
//     }
// }
//
// // Paste into EthercatDevice::output():
// //     let base = self.rx_bit_offset;
// //     output[base..(base + 16)].store_le::<u16>(self.rx_pdo.ao1);
// //     output[(base + 16)..(base + 32)].store_le::<u16>(self.rx_pdo.ao2);
// //     output[(base + 32)..(base + 48)].store_le::<u16>(self.rx_pdo.ao3);
// //     output[(base + 48)..(base + 64)].store_le::<u16>(self.rx_pdo.ao4);
// //
// // input_len: 0
// // output_len: 64  (channels * 16 bits)
// // input(): Ok(())
// // Struct fields: rx_pdo: Wago750_XXXRxPdo
// // NewEthercatDevice fields: rx_pdo: Wago750_XXXRxPdo::default()

// =============================================================================
// Main Device Struct
// =============================================================================
// Add the PDO fields from your chosen section's instructions.

#[derive(Clone)]
pub struct Wago750_XXX {
    is_used: bool,
    tx_bit_offset: usize,
    rx_bit_offset: usize,
    module: Option<Module>,
    // TODO: add PDO fields from your chosen section above
}

// =============================================================================
// Required Trait Implementations (always needed)
// =============================================================================
// Fill in input()/output()/input_len()/output_len() using the snippets
// from your chosen section's "Paste into" instructions above.

impl DynamicEthercatDevice for Wago750_XXX {}

impl EthercatDynamicPDO for Wago750_XXX {
    fn get_tx_offset(&self) -> usize {
        self.tx_bit_offset
    }

    fn get_rx_offset(&self) -> usize {
        self.rx_bit_offset
    }

    fn set_tx_offset(&mut self, offset: usize) {
        self.tx_bit_offset = offset
    }

    fn set_rx_offset(&mut self, offset: usize) {
        self.rx_bit_offset = offset
    }
}

impl EthercatDeviceUsed for Wago750_XXX {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl EthercatDevice for Wago750_XXX {
    fn input(
        &mut self,
        _input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        // TODO: paste input body from your chosen section
        Ok(())
    }

    fn input_len(&self) -> usize {
        todo!("Set input bit length (see section instructions)")
    }

    fn output(
        &self,
        _output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        // TODO: paste output body from your chosen section
        Ok(())
    }

    fn output_len(&self) -> usize {
        todo!("Set output bit length (see section instructions)")
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn is_module(&self) -> bool {
        true
    }

    fn input_checked(
        &mut self,
        _input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        // For analog inputs: self.input(_input)
        // For everything else: Ok(())
        Ok(())
    }

    fn output_checked(
        &self,
        _output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn get_module(&self) -> Option<Module> {
        self.module.clone()
    }

    fn set_module(&mut self, module: Module) {
        self.tx_bit_offset = module.tx_offset;
        self.rx_bit_offset = module.rx_offset;
        self.module = Some(module);
    }
}

impl EthercatDeviceProcessing for Wago750_XXX {}

impl NewEthercatDevice for Wago750_XXX {
    fn new() -> Self {
        Self {
            is_used: false,
            tx_bit_offset: 0,
            rx_bit_offset: 0,
            module: None,
            // TODO: add PDO defaults from your chosen section
        }
    }
}

impl std::fmt::Debug for Wago750_XXX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago750_XXX")
    }
}

// =============================================================================
// Device Identity
// =============================================================================

pub const WAGO_750_XXX_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_XXX_PRODUCT_ID: u32 = todo!("Set product ID from ESI file");
pub const WAGO_750_XXX_MODULE_IDENT: SubDeviceProductTuple =
    (WAGO_750_XXX_VENDOR_ID, WAGO_750_XXX_PRODUCT_ID);
