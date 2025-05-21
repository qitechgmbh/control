use std::{fs::File, io::Read};

use ethercat_hal::helpers::ethercrab_types::EthercrabSubDeviceGroupPreoperational;
use ethercrab::MainDevice;

use crate::{MAX_SUBDEVICES, PDI_LEN};

pub async fn restore_eeoprom(
    group: &EthercrabSubDeviceGroupPreoperational<MAX_SUBDEVICES, PDI_LEN>,
    maindevice: &MainDevice<'_>,
    subdevice_index: usize,
    file: &String,
) -> Result<(), anyhow::Error> {
    println!("Uploading EEPROM to subdevice {}...", subdevice_index);
    let subdevice = group.subdevice(maindevice, subdevice_index)?;
    let size = subdevice.eeprom_size(maindevice).await?;

    println!("Uploading {} bytes to EEPROM", size);

    let mut buffer: [u8; 2048] = [0; 2048];
    let mut file = File::open(file)?;
    Read::read_exact(&mut file, &mut buffer)?;

    // write the buffer
    for word in 0..(size / 2) {
        let byte = word * 2;
        let data = u16::from_le_bytes([buffer[byte], buffer[byte + 1]]);
        subdevice
            .eeprom_write_dangerously(maindevice, word as u16, data)
            .await?;
    }

    Ok(())
}
