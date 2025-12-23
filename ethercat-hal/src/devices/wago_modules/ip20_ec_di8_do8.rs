use super::*;
use crate::devices::{
    EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed, NewEthercatDevice,
    SubDeviceIdentityTuple,
};
use crate::{
    devices::{DynamicEthercatDevice, Module},
    helpers::ethercrab_types::EthercrabSubDevicePreoperational,
    io::{
        digital_input::{DigitalInputDevice, DigitalInputInput},
        digital_output::{DigitalOutputDevice, DigitalOutputOutput},
    },
};
use anyhow::Error;
use smol::lock::RwLock;
use std::sync::Arc;

const MODULE_COUNT_INDEX: (u16, u8) = (0xf050, 0x00);
const TX_MAPPING_INDEX: (u16, u8) = (0x1c13, 0x00);
const RX_MAPPING_INDEX: (u16, u8) = (0x1c12, 0x00);

/// Digital output port enumeration for the 8 outputs
#[derive(Debug, Clone, Copy)]
pub enum IP20EcDi8Do8OutputPort {
    DO1,
    DO2,
    DO3,
    DO4,
    DO5,
    DO6,
    DO7,
    DO8,
}

impl From<IP20EcDi8Do8OutputPort> for usize {
    fn from(value: IP20EcDi8Do8OutputPort) -> Self {
        match value {
            IP20EcDi8Do8OutputPort::DO1 => 0,
            IP20EcDi8Do8OutputPort::DO2 => 1,
            IP20EcDi8Do8OutputPort::DO3 => 2,
            IP20EcDi8Do8OutputPort::DO4 => 3,
            IP20EcDi8Do8OutputPort::DO5 => 4,
            IP20EcDi8Do8OutputPort::DO6 => 5,
            IP20EcDi8Do8OutputPort::DO7 => 6,
            IP20EcDi8Do8OutputPort::DO8 => 7,
        }
    }
}

/// Digital input port enumeration for the 8 inputs
#[derive(Debug, Clone, Copy)]
pub enum IP20EcDi8Do8InputPort {
    DI1,
    DI2,
    DI3,
    DI4,
    DI5,
    DI6,
    DI7,
    DI8,
}

impl From<IP20EcDi8Do8InputPort> for usize {
    fn from(value: IP20EcDi8Do8InputPort) -> Self {
        match value {
            IP20EcDi8Do8InputPort::DI1 => 0,
            IP20EcDi8Do8InputPort::DI2 => 1,
            IP20EcDi8Do8InputPort::DI3 => 2,
            IP20EcDi8Do8InputPort::DI4 => 3,
            IP20EcDi8Do8InputPort::DI5 => 4,
            IP20EcDi8Do8InputPort::DI6 => 5,
            IP20EcDi8Do8InputPort::DI7 => 6,
            IP20EcDi8Do8InputPort::DI8 => 7,
        }
    }
}

/// RX PDO structure (Master → Device, Outputs)
/// Mapped at 0x7000:01-08, 8 bits total
#[derive(Clone, Default)]
pub struct IP20EcDi8Do8RxPdo {
    pub do1: bool,
    pub do2: bool,
    pub do3: bool,
    pub do4: bool,
    pub do5: bool,
    pub do6: bool,
    pub do7: bool,
    pub do8: bool,
}

/// TX PDO structure (Device → Master, Inputs)
/// Mapped at 0x6000:01-08, 8 bits total
#[derive(Clone, Default)]
pub struct IP20EcDi8Do8TxPdo {
    pub di1: bool,
    pub di2: bool,
    pub di3: bool,
    pub di4: bool,
    pub di5: bool,
    pub di6: bool,
    pub di7: bool,
    pub di8: bool,
}

/// Wago 0x741:0x117b6722 bus coupler with digital I/O
/// This device has 8x Digital Input and 8x Digital Output terminals
pub struct IP20EcDi8Do8 {
    is_used: bool,
    pub slots: [Option<Module>; 64],
    pub slot_devices: [Option<Arc<RwLock<dyn DynamicEthercatDevice>>>; 64],
    pub dev_count: usize,
    pub module_count: usize,
    rx_offsets: Vec<usize>,
    tx_offsets: Vec<usize>,
    tx_size: usize,
    rx_size: usize,
    rx_pdo: IP20EcDi8Do8RxPdo,
    tx_pdo: IP20EcDi8Do8TxPdo,
}

impl EthercatDevice for IP20EcDi8Do8 {
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        // Read the 8 digital inputs (bits 0-7)
        if input.len() >= 8 {
            self.tx_pdo.di1 = input[0];
            self.tx_pdo.di2 = input[1];
            self.tx_pdo.di3 = input[2];
            self.tx_pdo.di4 = input[3];
            self.tx_pdo.di5 = input[4];
            self.tx_pdo.di6 = input[5];
            self.tx_pdo.di7 = input[6];
            self.tx_pdo.di8 = input[7];
        }

        // Handle slot devices if any
        for slot_device in &mut self.slot_devices {
            match slot_device {
                Some(device) => {
                    let mut d = device.write_blocking();
                    let _ = d.input(input);
                    drop(d);
                }
                None => break,
            }
        }

        Ok(())
    }

    fn input_len(&self) -> usize {
        if self.tx_size > 0 {
            self.tx_size
        } else {
            8 // 8 digital inputs
        }
    }

    fn output(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        // Write the 8 digital outputs (bits 0-7)
        if output.len() >= 8 {
            output.set(0, self.rx_pdo.do1);
            output.set(1, self.rx_pdo.do2);
            output.set(2, self.rx_pdo.do3);
            output.set(3, self.rx_pdo.do4);
            output.set(4, self.rx_pdo.do5);
            output.set(5, self.rx_pdo.do6);
            output.set(6, self.rx_pdo.do7);
            output.set(7, self.rx_pdo.do8);
        }

        // Handle slot devices if any
        for slot_device in &self.slot_devices {
            match slot_device {
                Some(device) => {
                    let d = device.read_blocking();
                    let _ = d.output(output);
                    drop(d);
                }
                None => break,
            }
        }
        Ok(())
    }

    fn output_len(&self) -> usize {
        if self.rx_size > 0 {
            self.rx_size
        } else {
            8 // 8 digital outputs
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn is_module(&self) -> bool {
        false
    }

    fn get_module(&self) -> Option<Module> {
        None
    }

    fn set_module(&mut self, module: Module) {
        self.slots[self.module_count] = Some(module);
        self.module_count += 1;
        self.tx_size += module.tx_offset;
        self.rx_size += module.rx_offset;
    }
}

impl EthercatDeviceUsed for IP20EcDi8Do8 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl EthercatDeviceProcessing for IP20EcDi8Do8 {}

impl NewEthercatDevice for IP20EcDi8Do8 {
    fn new() -> Self {
        Self {
            is_used: false,
            slots: [const { None }; 64],
            slot_devices: [const { None }; 64],
            module_count: 0,
            dev_count: 0,
            tx_size: 0,
            rx_size: 0,
            rx_offsets: vec![],
            tx_offsets: vec![],
            rx_pdo: IP20EcDi8Do8RxPdo::default(),
            tx_pdo: IP20EcDi8Do8TxPdo::default(),
        }
    }
}

impl std::fmt::Debug for IP20EcDi8Do8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WagoMaster_0x741_0x117b6722")
    }
}

impl DigitalInputDevice<IP20EcDi8Do8InputPort> for IP20EcDi8Do8 {
    fn get_input(&self, port: IP20EcDi8Do8InputPort) -> Result<DigitalInputInput, anyhow::Error> {
        Ok(DigitalInputInput {
            value: match port {
                IP20EcDi8Do8InputPort::DI1 => self.tx_pdo.di1,
                IP20EcDi8Do8InputPort::DI2 => self.tx_pdo.di2,
                IP20EcDi8Do8InputPort::DI3 => self.tx_pdo.di3,
                IP20EcDi8Do8InputPort::DI4 => self.tx_pdo.di4,
                IP20EcDi8Do8InputPort::DI5 => self.tx_pdo.di5,
                IP20EcDi8Do8InputPort::DI6 => self.tx_pdo.di6,
                IP20EcDi8Do8InputPort::DI7 => self.tx_pdo.di7,
                IP20EcDi8Do8InputPort::DI8 => self.tx_pdo.di8,
            },
        })
    }
}

impl DigitalOutputDevice<IP20EcDi8Do8OutputPort> for IP20EcDi8Do8 {
    fn set_output(&mut self, port: IP20EcDi8Do8OutputPort, value: DigitalOutputOutput) {
        let output_value: bool = value.into();
        match port {
            IP20EcDi8Do8OutputPort::DO1 => self.rx_pdo.do1 = output_value,
            IP20EcDi8Do8OutputPort::DO2 => self.rx_pdo.do2 = output_value,
            IP20EcDi8Do8OutputPort::DO3 => self.rx_pdo.do3 = output_value,
            IP20EcDi8Do8OutputPort::DO4 => self.rx_pdo.do4 = output_value,
            IP20EcDi8Do8OutputPort::DO5 => self.rx_pdo.do5 = output_value,
            IP20EcDi8Do8OutputPort::DO6 => self.rx_pdo.do6 = output_value,
            IP20EcDi8Do8OutputPort::DO7 => self.rx_pdo.do7 = output_value,
            IP20EcDi8Do8OutputPort::DO8 => self.rx_pdo.do8 = output_value,
        }
    }

    fn get_output(&self, port: IP20EcDi8Do8OutputPort) -> DigitalOutputOutput {
        let current_value = match port {
            IP20EcDi8Do8OutputPort::DO1 => self.rx_pdo.do1,
            IP20EcDi8Do8OutputPort::DO2 => self.rx_pdo.do2,
            IP20EcDi8Do8OutputPort::DO3 => self.rx_pdo.do3,
            IP20EcDi8Do8OutputPort::DO4 => self.rx_pdo.do4,
            IP20EcDi8Do8OutputPort::DO5 => self.rx_pdo.do5,
            IP20EcDi8Do8OutputPort::DO6 => self.rx_pdo.do6,
            IP20EcDi8Do8OutputPort::DO7 => self.rx_pdo.do7,
            IP20EcDi8Do8OutputPort::DO8 => self.rx_pdo.do8,
        };
        DigitalOutputOutput(current_value)
    }
}

impl IP20EcDi8Do8 {
    pub async fn get_pdo_offsets<'a>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'a>,
        get_tx: bool,
    ) -> Result<(), Error> {
        let mut vec: Vec<usize> = vec![];
        let mut bit_offset = 0;
        let start_subindex = 0x1;
        let index = match get_tx {
            true => (TX_MAPPING_INDEX.0, TX_MAPPING_INDEX.1),
            false => (RX_MAPPING_INDEX.0, RX_MAPPING_INDEX.1),
        };
        let count = device.sdo_read::<u8>(index.0, index.1).await?;

        for i in 0..count {
            vec.push(bit_offset);
            let pdo_index = device.sdo_read(index.0, start_subindex + i).await?;
            if pdo_index != 0 {
                let pdo_map_count = device.sdo_read::<u8>(pdo_index, 0).await?;
                // Iterate over every PDO Mapped entry and add it to the cumulative bit offset
                for j in 0..pdo_map_count {
                    let pdo_mapping: u32 = device.sdo_read(pdo_index, 1 + j).await?;
                    // We only need / Want the bit len, which we extract with a bitmask extracting the lsb
                    let bit_length = (pdo_mapping & 0xFF) as u8;
                    bit_offset += bit_length as usize;
                }
            }
        }

        if get_tx {
            self.tx_offsets = vec;
        } else {
            self.rx_offsets = vec;
        }
        Ok(())
    }

    pub async fn get_module_count<'a>(
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<usize, Error> {
        match device
            .sdo_read::<u8>(MODULE_COUNT_INDEX.0, MODULE_COUNT_INDEX.1)
            .await
        {
            Ok(value) => Ok(value as usize),
            Err(_e) => Ok(0),
        }
    }

    pub async fn get_modules<'a>(
        device: &EthercrabSubDevicePreoperational<'a>,
        module_count: usize,
    ) -> Result<Vec<crate::devices::Module>, Error> {
        const MODULES_START_ADDR: u16 = 0x9000;
        const MODULE_IDENT_SUBINDEX: u8 = 0x0a;
        let mut modules: Vec<Module> = vec![];

        for i in 0..module_count {
            let module_addr = MODULES_START_ADDR + (i * 0x10) as u16;
            let ident_iom = device
                .sdo_read::<u32>(module_addr, MODULE_IDENT_SUBINDEX)
                .await?;

            // For Wago the IOM will be the product ID
            let mut module = Module {
                slot: i as u16,
                belongs_to_addr: device.configured_address(),
                vendor_id: device.identity().vendor_id,
                product_id: ident_iom,
                has_tx: false,
                has_rx: false,
                tx_offset: 0,
                rx_offset: 0,
            };

            match ident_iom {
                // Add Module idents here when discovered
                wago_750_1506::WAGO_750_1506_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = true;
                }
                wago_750_501::WAGO_750_501_PRODUCT_ID => {
                    module.has_tx = false;
                    module.has_rx = true;
                }
                wago_750_652::WAGO_750_652_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = true;
                }
                _ => {}
            }
            modules.push(module);
        }
        Ok(modules)
    }

    /// Call after all modules have been added
    pub fn init_slot_modules<'a>(&mut self, device: &EthercrabSubDevicePreoperational<'a>) {
        // Already initialized
        if self.dev_count != 0 {
            return;
        }
        let mut tx_index = 1;
        let mut rx_index = 1;

        smol::block_on(async {
            let _ = self.get_pdo_offsets(device, true).await;
            let _ = self.get_pdo_offsets(device, false).await;
        });

        for module in self.slots {
            match module {
                Some(m) => {
                    // Map ModuleIdent's to Terminals
                    let dev: Arc<RwLock<dyn DynamicEthercatDevice>> =
                        match (m.vendor_id, m.product_id) {
                            wago_750_501::WAGO_750_501_MODULE_IDENT => {
                                Arc::new(RwLock::new(wago_750_501::Wago750_501::new()))
                            }
                            wago_750_1506::WAGO_750_1506_MODULE_IDENT => {
                                Arc::new(RwLock::new(wago_750_1506::Wago750_1506::new()))
                            }
                            wago_750_652::WAGO_750_652_MODULE_IDENT => {
                                Arc::new(RwLock::new(wago_750_652::Wago750_652::new()))
                            }
                            _ => {
                                return;
                            }
                        };

                    let tx_pdo_offset = self.tx_offsets.get(tx_index as usize);
                    if m.has_tx {
                        tx_index += 1;
                    }

                    let rx_pdo_offset = self.rx_offsets.get(rx_index as usize);
                    if m.has_rx {
                        rx_index += 1;
                    }

                    let mut dev_guard = dev.write_blocking();
                    match tx_pdo_offset {
                        Some(offset) => dev_guard.set_tx_offset(*offset),
                        None => (),
                    }

                    match rx_pdo_offset {
                        Some(offset) => dev_guard.set_rx_offset(*offset),
                        None => (),
                    }
                    drop(dev_guard);
                    self.slot_devices[self.dev_count] = Some(dev);
                    self.dev_count += 1;
                }
                None => break,
            }
        }
    }

    pub async fn initialize_modules<'a>(
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<Vec<Module>, Error> {
        let count = match IP20EcDi8Do8::get_module_count(device).await {
            Ok(count) => count,
            Err(e) => return Err(e),
        };
        if count == 0 {
            return Ok(vec![]);
        }
        let modules = IP20EcDi8Do8::get_modules(device, count).await?;
        Ok(modules)
    }
}

pub const IP20_EC_DI8_DO8_VENDOR_ID: u32 = 0x741;
pub const IP20_EC_DI8_DO8_PRODUCT_ID: u32 = 0x117b6722;
pub const IP20_EC_DI8_DO8_REVISION: u32 = 0x1;
pub const IP20_EC_DI8_DO8_IDENTITY: SubDeviceIdentityTuple = (
    IP20_EC_DI8_DO8_VENDOR_ID,
    IP20_EC_DI8_DO8_PRODUCT_ID,
    IP20_EC_DI8_DO8_REVISION,
);
