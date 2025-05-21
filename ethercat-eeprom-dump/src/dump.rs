use std::{fs::File, io::Write};

use ethercat_hal::helpers::ethercrab_types::EthercrabSubDeviceGroupPreoperational;
use ethercrab::MainDevice;

use crate::{MAX_SUBDEVICES, PDI_LEN, print::print_buffer};

pub async fn dump_eeprom(
    group: &EthercrabSubDeviceGroupPreoperational<MAX_SUBDEVICES, PDI_LEN>,
    maindevice: &MainDevice<'_>,
    subdevice_index: usize,
    file: Option<&String>,
) -> Result<(), anyhow::Error> {
    println!("Reading EEPROM from subdevice {}...", subdevice_index);
    let subdevice = group.subdevice(maindevice, subdevice_index)?;
    let size = subdevice.eeprom_size(maindevice).await?;

    println!("Reading {} bytes from EEPROM", size);

    let mut buffer: [u8; 2048] = [0; 2048];
    subdevice
        .eeprom_read_raw(maindevice, 0, &mut buffer)
        .await?;

    // if file is Some write to file
    if let Some(file) = file {
        let mut file = File::create(file)?;
        Write::write_all(&mut file, &buffer)?;
    } else {
        // print the buffer
        print_buffer(buffer.as_slice());
    }

    Ok(())
}
