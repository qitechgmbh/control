pub mod counter_wrapper_u16_i128;
pub mod el70xx_velocity_converter;
pub mod ethercrab_types;
pub mod signing_converter_u16;

use ethercrab::{SubDevice, SubDeviceRef};
// This is the Codeword for disabling EEPROM writes on sdo writes
const BECKHOFF_EEPROM_LOCK_CODEWORD : u32 = 0x12345678;
// In this index Codewords can be written for Beckhoff Terminals
const BECKHOFF_CODEWORD_INDEX : u16 = 0xF008;

// This code locks the eeprom and protects it from writes
// When the toggle is in effect no writes propagate to the eeprom 
// This is useful if you have a usecase where you very often write into CoE registers, as writes to EEPROM after a long time degrage the EEPROM
pub async fn set_mut_beckhoff_eeprom_lock_active<'a>(subdevice : &SubDeviceRef<'a, &mut SubDevice>) -> Result<(),anyhow::Error>{
    let code_word = match subdevice.sdo_read::<u32>(BECKHOFF_CODEWORD_INDEX,0).await {
        Ok(code_word) => code_word,
        // This happens when the subdevice has no mailbox
        // There is NO check in ethercrab for Mailbox presence, so we just have to pray that it has one and send a request
        // If there is no Mailbox to write Coe Request to, then there is also no EEPROM meaning we achieved our goal in a sense
        // This is why it returns OK on an error for sdo_read
        Err(_) => return Ok(()),
    };

    let eeprom_lock_toggled = match code_word {
        BECKHOFF_EEPROM_LOCK_CODEWORD => true,
        _ => false,
    };

    if !eeprom_lock_toggled {
        subdevice.sdo_write(BECKHOFF_CODEWORD_INDEX,0,BECKHOFF_EEPROM_LOCK_CODEWORD).await?;
        tracing::info!("Activated EEPROM Lock for {} {}",subdevice.name(),subdevice.name());
    }

    Ok(())
}

pub async fn set_beckhoff_eeprom_lock_active<'a>(subdevice : &SubDeviceRef<'a, &SubDevice>) -> Result<(),anyhow::Error>{
    let code_word = match subdevice.sdo_read::<u32>(BECKHOFF_CODEWORD_INDEX,0).await {
        Ok(code_word) => code_word,
        // This happens when the subdevice has no mailbox
        // There is NO check in ethercrab for Mailbox presence, so we just have to pray that it has one and send a request
        // If there is no Mailbox to write Coe Request to, then there is also no EEPROM meaning we achieved our goal in a sense
        // This is why it returns OK on an error for sdo_read
        Err(_) => return Ok(()),
    };

    let eeprom_lock_toggled = match code_word {
        BECKHOFF_EEPROM_LOCK_CODEWORD => true,
        _ => false,
    };

    if !eeprom_lock_toggled {
        subdevice.sdo_write(BECKHOFF_CODEWORD_INDEX,0,BECKHOFF_EEPROM_LOCK_CODEWORD).await?;
        tracing::info!("Activated EEPROM Lock for {} {}",subdevice.name(),subdevice.name());
    }

    Ok(())
}