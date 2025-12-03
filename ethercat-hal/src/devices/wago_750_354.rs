use std::sync::Arc;
use crate::{devices::{DynamicEthercatDevice, wago_modules::{wago_750_501::{WAGO_750_501_MODULE_IDENT, WAGO_750_501_PRODUCT_ID}, wago_750_652::{WAGO_750_652_MODULE_IDENT, WAGO_750_652_PRODUCT_ID}, wago_750_1506::{WAGO_750_1506_MODULE_IDENT, WAGO_750_1506_PRODUCT_ID}}}, helpers::ethercrab_types::EthercrabSubDevicePreoperational};

use super::{EthercatDevice, EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use anyhow::Error;
use ethercat_hal_derive::EthercatDevice;
use smol::lock::RwLock;
const MODULE_COUNT_INDEX : (u16,u8) = (0xf050,0x00);
const TX_MAPPING_INDEX : (u16,u8) = (0x1c13,0x00);  
const RX_MAPPING_INDEX : (u16,u8) = (0x1c12,0x00);  
use crate::devices::wago_modules::*;

/// Wago750_354 bus coupler
#[derive(Clone, EthercatDevice)]
pub struct Wago750_354 {
    is_used: bool,
}

impl EthercatDeviceProcessing for Wago750_354 {}

impl NewEthercatDevice for Wago750_354 {
    fn new() -> Self {
        Self { is_used: false }
    }
}

impl std::fmt::Debug for Wago750_354 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {        
        write!(f, "Wago_750_354")
    }
}


#[derive(Debug)]
pub struct Module {
    pub slot : u16,
    pub belongs_to_addr : u16,
    pub has_tx : bool,
    pub has_rx : bool,
    pub vendor_id : u32,
    pub product_id : u32,
}

impl Wago750_354 {
    pub async fn get_pdo_offsets<'a>(device: &EthercrabSubDevicePreoperational<'a>,get_tx : bool) -> Result<Vec<usize>,Error> {
        let mut vec : Vec<usize>= vec![];
        let mut bit_offset = 0;
        let start_subindex = 0x1;
        let index = match get_tx {
            true => (TX_MAPPING_INDEX.0, TX_MAPPING_INDEX.1),
            false => (RX_MAPPING_INDEX.0, RX_MAPPING_INDEX.1),
        };
        let count = device.sdo_read::<u8>(index.0, index.1).await?;

        for i in 0..count {
            vec.push(bit_offset);
            let pdo_index = device.sdo_read(index.0, start_subindex+i).await?;
            if pdo_index != 0 {
                let pdo_map_count = device.sdo_read::<u8>(pdo_index, 0).await?;
                
                // Iterate over every PDO Mapped entry and add it to the cumulative bit offset
                for j in 0..pdo_map_count {
                    let pdo_mapping : u32 = device.sdo_read(pdo_index, 1 + j).await?;
                    // We only need / Want the bit len
                    let bit_length = (pdo_mapping & 0xFF) as u8;
                    bit_offset += bit_length as usize;
                } 
            }
        }

        tracing::info!("{} is tx: {}",count,get_tx);
        Ok(vec)   
    }

    // The Coupler gets itself as a Preop Subdevice
    pub async fn get_module_count<'a>(device: &EthercrabSubDevicePreoperational<'a>) -> Result<usize,Error> {
        match device.sdo_read::<u8>(MODULE_COUNT_INDEX.0, MODULE_COUNT_INDEX.1).await {
            Ok(value) => Ok(value as usize),
            Err(e) => Err(anyhow::anyhow!("Failed to read Module Count for Wago750_354: {:?}",e)),
        }
    }

    pub async fn get_modules<'a>(device: &EthercrabSubDevicePreoperational<'a>, module_count : usize) -> Result<Vec<Module>,Error> {
        const MODULES_START_ADDR : u16 = 0x9000;
        const MODULE_IDENT_SUBINDEX : u8 = 0x0a;

        let mut modules : Vec<Module> = vec![];
        
        for i in 0..module_count {
            let module_addr = MODULES_START_ADDR + (i * 0x10) as u16;            
            
            let ident_iom = device.sdo_read::<u32>(module_addr, MODULE_IDENT_SUBINDEX).await?;
            /*
                For Wago the IOM is also the product ID
            */
            let mut module = Module{
                slot:i as u16, 
                belongs_to_addr: device.configured_address(),
                vendor_id:  device.identity().vendor_id, 
                product_id: ident_iom,
                has_tx: false,
                has_rx: false, 
            };

            match ident_iom {
                WAGO_750_1506_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = true;
                },
                WAGO_750_501_PRODUCT_ID => {
                    module.has_tx = false;
                    module.has_rx = true;
                },
                WAGO_750_652_PRODUCT_ID => {
                    module.has_tx = true;
                    module.has_rx = true;
                },
                _ => (),
            }
            modules.push(module);   
        }
        Ok(modules)
    }

    pub async fn subdevices_from_modules(tx_pdos : &Vec<usize>, rx_pdos : &Vec<usize>,modules : &Vec<Module>) -> Vec<Arc<RwLock<dyn EthercatDevice>>> {
        let mut devices = vec![];
        let mut tx_index = 1;
        let mut rx_index = 1;

        for module in modules.iter() {      
            let tx_pdo_offset = tx_pdos.get(tx_index as usize);
            if module.has_tx {
                tx_index += 1;
            }

            let rx_pdo_offset = rx_pdos.get(rx_index as usize);
            if module.has_rx {
                rx_index += 1;
            }
            
            let device : Arc<RwLock<dyn EthercatDevice>> = match (module.vendor_id,module.product_id) {
                WAGO_750_501_MODULE_IDENT => Arc::new(RwLock::new(wago_750_501::Wago750_501::new())),
                WAGO_750_1506_MODULE_IDENT => Arc::new(RwLock::new(wago_750_1506::Wago750_1506::new())),
                WAGO_750_652_MODULE_IDENT => Arc::new(RwLock::new(wago_750_652::Wago750_652::new())),
                _ => return  devices,   
            };

            if let Some(dev) = device.write().await.as_any_mut().downcast_mut::<Box<dyn DynamicEthercatDevice>>() {
                match tx_pdo_offset {
                    Some(offset) => dev.set_tx_offset(*offset),
                    None => (),
                }

                match rx_pdo_offset {
                    Some(offset) => dev.set_rx_offset(*offset),
                    None => (),
                }

            }
            devices.push(device);
        }
        return devices;
    }

    pub async fn initialize_modules<'a> (device: &EthercrabSubDevicePreoperational<'a>) -> Result<Vec<Arc<RwLock<dyn EthercatDevice>>>,Error> {
        let count = match Wago750_354::get_module_count(device).await {
            Ok(count) => count,
            Err(e) => return Err(e),
        };

        if count == 0 {
            return Ok(vec![]);
        }

        let tx_offsets = Wago750_354::get_pdo_offsets(device,true).await?;
        let rx_offsets = Wago750_354::get_pdo_offsets(device,false).await?;
        
        let modules = Wago750_354::get_modules(device,count).await?;        
        let devices =  Wago750_354::subdevices_from_modules(&tx_offsets,&rx_offsets,&modules).await; 

        println!("{:?}",tx_offsets);
        println!("{:?}",rx_offsets);
        println!("{:?}",modules);

        Ok(devices)
    }
}




pub const WAGO_750_354_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_354_PRODUCT_ID: u32 = 0x07500354;
pub const WAGO_750_354_REVISION_A: u32 = 0x2;
pub const WAGO_750_354_IDENTITY_A: SubDeviceIdentityTuple =
    (WAGO_750_354_VENDOR_ID, WAGO_750_354_PRODUCT_ID, WAGO_750_354_REVISION_A);
