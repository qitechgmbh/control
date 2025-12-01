use super::{
    EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed, NewEthercatDevice,
    SubDeviceIdentityTuple,
};
use crate::devices::wago_modules::*;
use crate::{
    devices::{
        DynamicEthercatDevice, Module,
        wago_modules::{
            wago_750_501::{WAGO_750_501_MODULE_IDENT, WAGO_750_501_PRODUCT_ID},
            wago_750_652::{WAGO_750_652_MODULE_IDENT, WAGO_750_652_PRODUCT_ID},
            wago_750_1506::{WAGO_750_1506_MODULE_IDENT, WAGO_750_1506_PRODUCT_ID},
        },
    },
    helpers::ethercrab_types::EthercrabSubDevicePreoperational,
};
use anyhow::Error;
use smol::lock::RwLock;
use std::sync::Arc;

const MODULE_COUNT_INDEX: (u16, u8) = (0xf050, 0x00);
const TX_MAPPING_INDEX: (u16, u8) = (0x1c13, 0x00);
const RX_MAPPING_INDEX: (u16, u8) = (0x1c12, 0x00);
// For both the rx and tx The Wago Coupler has 4 bytes, which we dont care about and skip

/// Wago750_354 bus coupler
/*
    The "Modules" simply write at an offset into rx and read at an offset in tx
*/
pub struct Wago750_354 {
    is_used: bool,
    pub slots: [Option<Module>; 64],
    pub slot_devices: [Option<Arc<RwLock<dyn DynamicEthercatDevice>>>; 64],
    pub dev_count: usize,
    pub module_count: usize,
    rx_offsets: Vec<usize>,
    tx_offsets: Vec<usize>,
    tx_size: usize,
    rx_size: usize,
}

impl EthercatDevice for Wago750_354 {
    fn input(
        &mut self,
        input: &bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        for slot_device in &mut self.slot_devices {
            match slot_device {
                Some(device) => {
                    // Give all Modules access to the couplers input image and call their input func
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
        self.tx_size
    }

    fn output(
        &self,
        output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>,
    ) -> Result<(), anyhow::Error> {
        for slot_device in &self.slot_devices {
            match slot_device {
                Some(device) => {
                    // Give all Modules access to the couplers Output image and call their output func
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
        self.rx_size
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

impl EthercatDeviceUsed for Wago750_354 {
    fn is_used(&self) -> bool {
        self.is_used
    }

    fn set_used(&mut self, used: bool) {
        self.is_used = used;
    }
}

impl EthercatDeviceProcessing for Wago750_354 {}

impl NewEthercatDevice for Wago750_354 {
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
        }
    }
}

impl std::fmt::Debug for Wago750_354 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago_750_354")
    }
}

impl Wago750_354 {
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
            Err(e) => Err(anyhow::anyhow!(
                "Failed to read Module Count for Wago750_354: {:?}",
                e
            )),
        }
    }

    // This should probably be a generic function instead
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
            // For Wago the IOM well be the product ID
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
                // Add Module idents here:
                WAGO_750_1506_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = true;
                }
                WAGO_750_501_PRODUCT_ID => {
                    module.has_tx = false;
                    module.has_rx = true;
                }
                WAGO_750_652_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = true;
                }
                _ => println!(
                    "Wago-750-354 found Unknown/Unimplemented Module: {}",
                    ident_iom
                ),
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
                    let dev: Arc<RwLock<dyn DynamicEthercatDevice>> = match (
                        m.vendor_id,
                        m.product_id,
                    ) {
                        WAGO_750_501_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_501::Wago750_501::new()))
                        }
                        WAGO_750_1506_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_1506::Wago750_1506::new()))
                        }
                        WAGO_750_652_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_652::Wago750_652::new()))
                        }
                        _ => {
                            println!(
                                "{} Missing Implementation for Module Identification: vendor_id: {:?}, module ident: {:?} !",
                                module_path!(),
                                m.vendor_id,
                                m.product_id
                            );
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
        let count = match Wago750_354::get_module_count(device).await {
            Ok(count) => count,
            Err(e) => return Err(e),
        };
        if count == 0 {
            return Ok(vec![]);
        }
        let modules = Wago750_354::get_modules(device, count).await?;
        Ok(modules)
    }
}

pub const WAGO_750_354_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_354_PRODUCT_ID: u32 = 0x07500354;
pub const WAGO_750_354_REVISION_A: u32 = 0x2;
pub const WAGO_750_354_IDENTITY_A: SubDeviceIdentityTuple = (
    WAGO_750_354_VENDOR_ID,
    WAGO_750_354_PRODUCT_ID,
    WAGO_750_354_REVISION_A,
);
