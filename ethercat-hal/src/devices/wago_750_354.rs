use std::sync::Arc;
use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;

use super::{EthercatDevice, EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use anyhow::Error;
use ethercat_hal_derive::EthercatDevice;
use smol::lock::RwLock;


const MODULE_COUNT_INDEX : (u16,u8) = (0xf050,0x00);

const TX_MAPPING_INDEX : (u16,u8) = (0x1c13,0x00);  
const RX_MAPPING_INDEX : (u16,u8) = (0x1c12,0x00);  

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
    pub belongs_to_slave_index : u16,
    pub has_tx : bool,
    pub has_rx : bool,
    pub vendor_id : u32,
    pub product_id : u32,
    pub name : String,
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
        let modules : Vec<Module> = vec![];
        for i in 0..module_count {
            let module_addr = MODULES_START_ADDR + (i * 0x10) as u16;
            let ident = device.sdo_read::<u32>(module_addr, 0x00).await?;
            device.sdo_read(index, sub_index).await?;
        }
        Ok(vec![])
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

        println!("{:?}",tx_offsets);
        println!("{:?}",rx_offsets);
        Ok(vec![])
    }
}




pub const WAGO_750_354_VENDOR_ID: u32 = 0x00000021;
pub const WAGO_750_354_PRODUCT_ID: u32 = 0x07500354;
pub const WAGO_750_354_REVISION_A: u32 = 0x2;
pub const WAGO_750_354_IDENTITY_A: SubDeviceIdentityTuple =
    (WAGO_750_354_VENDOR_ID, WAGO_750_354_PRODUCT_ID, WAGO_750_354_REVISION_A);
