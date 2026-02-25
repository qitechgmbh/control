use super::{
    EthercatDevice, EthercatDeviceProcessing, EthercatDeviceUsed, NewEthercatDevice,
    SubDeviceIdentityTuple,
};
use crate::devices::wago_modules::wago_750_430::{
    WAGO_750_430_MODULE_IDENT, WAGO_750_430_PRODUCT_ID,
};
use crate::devices::wago_modules::*;
use crate::{
    devices::{
        DynamicEthercatDevice, Module,
        wago_modules::{
            wago_750_402::{WAGO_750_402_MODULE_IDENT, WAGO_750_402_PRODUCT_ID},
            wago_750_455::{WAGO_750_455_MODULE_IDENT, WAGO_750_455_PRODUCT_ID},
            wago_750_501::{WAGO_750_501_MODULE_IDENT, WAGO_750_501_PRODUCT_ID},
            wago_750_530::{WAGO_750_530_MODULE_IDENT, WAGO_750_530_PRODUCT_ID},
            wago_750_652::{WAGO_750_652_MODULE_IDENT, WAGO_750_652_PRODUCT_ID},
            wago_750_672::{WAGO_750_672_MODULE_IDENT, WAGO_750_672_PRODUCT_ID},
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

#[derive(Clone, Debug)]
struct ModulePdoMapping {
    pub offset: usize,
    pub module_i: u32,
}

// For both the rx and tx The Wago Coupler has 4 bytes, which we dont care about and skip
pub struct Wago750_354 {
    is_used: bool,
    pub slots: [Option<Module>; 64],
    pub slot_devices: [Option<Arc<RwLock<dyn DynamicEthercatDevice>>>; 64],
    pub dev_count: usize,
    pub module_count: usize,
    rx_pdo_mappings: Vec<ModulePdoMapping>,
    tx_pdo_mappings: Vec<ModulePdoMapping>,
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
        self.slots[self.module_count] = Some(module.clone());
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
            rx_pdo_mappings: vec![],
            tx_pdo_mappings: vec![],
        }
    }
}

impl std::fmt::Debug for Wago750_354 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wago_750_354")
    }
}

impl Wago750_354 {
    pub fn calculate_module_index(pdo_mapping: u32, is_tx: bool) -> u32 {
        let start_index = match is_tx {
            true => 0x6000,
            false => 0x7000,
        };
        let pdo_index = (pdo_mapping & 0xFFFF0000) >> 16;

        if pdo_index < start_index {
            // Treat as coupler/module 0
            return 0;
        }

        let index_in_hex = pdo_index - start_index;

        if index_in_hex < 16 {
            0
        } else {
            index_in_hex / 16
        }
    }

    pub async fn get_pdo_offsets<'a>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'a>,
        get_tx: bool,
    ) -> Result<(), Error> {
        let mut vec: Vec<ModulePdoMapping> = vec![];
        let mut bit_offset = 0;

        let mut module_i;
        let start_subindex = 0x2;

        let index = match get_tx {
            true => (TX_MAPPING_INDEX.0, TX_MAPPING_INDEX.1),
            false => (RX_MAPPING_INDEX.0, RX_MAPPING_INDEX.1),
        };

        let count_mappings = device.sdo_read::<u8>(index.0, index.1).await?;
        let pdo_index = device.sdo_read::<u16>(index.0, 1).await?;
        let pdo_map_count = device.sdo_read::<u8>(pdo_index, 0).await?;

        for i in 0..pdo_map_count {
            let pdo_mapping: u32 = device.sdo_read(pdo_index, 1 + i).await?;
            let bit_length = (pdo_mapping & 0xFF) as u8;
            bit_offset += bit_length as usize;
        }

        let mut mappings_without_coupler: Vec<u32> = vec![];
        for i in start_subindex..=count_mappings {
            let pdo_index = device.sdo_read(index.0, i).await?;
            let pdo_map_count = device.sdo_read::<u8>(pdo_index, 0).await?;
            for j in 0..pdo_map_count {
                let pdo_mapping: u32 = device.sdo_read(pdo_index, 1 + j).await?;
                mappings_without_coupler.push(pdo_mapping);
            }
        }
        mappings_without_coupler.sort();

        for pdo_mapping in mappings_without_coupler {
            module_i = Wago750_354::calculate_module_index(pdo_mapping, get_tx);
            let bit_length = (pdo_mapping & 0xFF) as u8;
            if module_i < 64 {
                vec.push(ModulePdoMapping {
                    offset: bit_offset,
                    module_i,
                });
            }
            bit_offset += bit_length as usize;
        }

        vec.sort_by_key(|e| (e.module_i, e.offset));
        // deduplicate by module_i, so we only have the offset to the start of inputs/outputs
        vec.dedup_by(|a, b| a.module_i == b.module_i);

        if get_tx {
            self.tx_pdo_mappings = vec;
        } else {
            self.rx_pdo_mappings = vec;
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
                name: "".to_string(),
            };

            match ident_iom {
                // Add Module idents here:
                WAGO_750_1506_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = true;
                    module.name = "750-1506".to_string();
                }
                WAGO_750_455_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = false;
                    module.name = "750-455".to_string();
                }
                WAGO_750_501_PRODUCT_ID => {
                    module.has_tx = false;
                    module.has_rx = true;
                    module.name = "750-501".to_string();
                }
                WAGO_750_530_PRODUCT_ID => {
                    module.has_tx = false;
                    module.has_rx = true;
                    module.name = "750-530".to_string();
                }
                WAGO_750_652_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = true;
                    module.name = "750-652".to_string();
                }
                WAGO_750_402_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = false;
                    module.name = "750-402".to_string();
                }
                WAGO_750_430_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = false;
                    module.name = "750-430".to_string();
                }
                WAGO_750_672_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = true;
                    module.name = "750-672".to_string();
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

        smol::block_on(async {
            let _ = self.get_pdo_offsets(device, true).await;
            let _ = self.get_pdo_offsets(device, false).await;
        });

        for module in &mut self.slots {
            match module {
                Some(m) => {
                    let tx_pdo_mapping = self
                        .tx_pdo_mappings
                        .iter()
                        .find(|map| map.module_i == m.slot.into());
                    if m.has_tx {
                        m.tx_offset = match tx_pdo_mapping {
                            Some(map) => map.offset,
                            None => 0,
                        }
                    }

                    let rx_pdo_mapping = self
                        .rx_pdo_mappings
                        .iter()
                        .find(|map| map.module_i == m.slot.into());
                    if m.has_rx {
                        m.rx_offset = match rx_pdo_mapping {
                            Some(map) => map.offset,
                            None => 0,
                        }
                    }
                }
                None => break,
            }
        }

        for module in &self.slots {
            match module {
                Some(m) => {
                    // Map ModuleIdent's to Terminals
                    let dev: Arc<RwLock<dyn DynamicEthercatDevice>> = match (
                        m.vendor_id,
                        m.product_id,
                    ) {
                        WAGO_750_455_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_455::Wago750_455::new()))
                        }
                        WAGO_750_501_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_501::Wago750_501::new()))
                        }
                        WAGO_750_530_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_530::Wago750_530::new()))
                        }
                        WAGO_750_1506_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_1506::Wago750_1506::new()))
                        }
                        WAGO_750_652_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_652::Wago750_652::new()))
                        }
                        WAGO_750_402_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_402::Wago750_402::new()))
                        }
                        WAGO_750_430_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_430::Wago750_430::new()))
                        }
                        WAGO_750_672_MODULE_IDENT => {
                            Arc::new(RwLock::new(wago_750_672::Wago750_672::new()))
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
                    let mut dev_guard = dev.write_blocking();
                    //println!("For {:?} setting tx: {} rx: {}",m.name,m.tx_offset,m.rx_offset);
                    dev_guard.set_tx_offset(m.tx_offset);
                    dev_guard.set_rx_offset(m.rx_offset);
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
