use anyhow::anyhow;
use control_core::ethercat::eeprom_identification::{
    get_identification_addresses, machine_device_identification,
};
use ethercat_hal::{
    devices::device_from_subdevice_identity,
    helpers::ethercrab_types::EthercrabSubDeviceGroupPreoperational,
};
use ethercrab::MainDevice;
use smol;

use crate::{MAX_SUBDEVICES, PDI_LEN, print::print_markdown_table};

pub fn ls(
    group: EthercrabSubDeviceGroupPreoperational<MAX_SUBDEVICES, PDI_LEN>,
    maindevice: &MainDevice,
) {
    // print a table of all devices
    let mut table = vec![vec![
        "Index".to_string(),
        "Name".to_string(),
        "Vendor (EC)".to_string(),
        "Product (EC)".to_string(),
        "Revision (EC)".to_string(),
        "Serial (EC)".to_string(),
        "ethercat-hal".to_string(),
        "Vendor (MID)".to_string(),
        "Machine (MID)".to_string(),
        "Serial (MID)".to_string(),
        "Role (MID)".to_string(),
    ]];
    for (i, device) in group.iter(maindevice).enumerate() {
        let identity = device.identity();
        let driver = device_from_subdevice_identity(&identity);
        let identification_adresses = get_identification_addresses(&identity, device.name());
        let identification = match identification_adresses {
            Ok(_) => smol::block_on(machine_device_identification(&device, maindevice)),
            Err(_) => Err(anyhow!("")),
        };

        table.push(vec![
            i.to_string(),
            device.name().to_string(),
            format!("0x{:x}", identity.vendor_id),
            format!("0x{:x}", identity.product_id),
            format!("0x{:x}", identity.revision),
            format!("0x{:x}", identity.serial),
            match driver {
                Ok(_) => "Yes".to_string(),
                Err(_) => "No".to_string(),
            },
            // vendor identification
            format!(
                "{} @ {}",
                match &identification {
                    Ok(identification) => format!(
                        "0x{:x}",
                        identification
                            .machine_identification_unique
                            .machine_identification
                            .vendor
                    ),
                    Err(_) => "-".to_string(),
                },
                match &identification_adresses {
                    Ok(identification_adresses) =>
                        format!("0x{:x}", identification_adresses.vendor_word),
                    Err(_) => "-".to_string(),
                }
            ),
            // machine identification
            format!(
                "{} @ {}",
                match &identification {
                    Ok(identification) => format!(
                        "0x{:x}",
                        identification
                            .machine_identification_unique
                            .machine_identification
                            .machine
                    ),
                    Err(_) => "-".to_string(),
                },
                match &identification_adresses {
                    Ok(identification_adresses) =>
                        format!("0x{:x}", identification_adresses.machine_word),
                    Err(_) => "-".to_string(),
                }
            ),
            // machine serial
            format!(
                "{} @ {}",
                match &identification {
                    Ok(identification) => format!(
                        "0x{:x}",
                        identification.machine_identification_unique.serial
                    ),
                    Err(_) => "-".to_string(),
                },
                match &identification_adresses {
                    Ok(identification_adresses) =>
                        format!("0x{:x}", identification_adresses.serial_word),
                    Err(_) => "-".to_string(),
                }
            ),
            // device role
            format!(
                "{} @ {}",
                match &identification {
                    Ok(identification) => format!("0x{:x}", identification.role),
                    Err(_) => "-".to_string(),
                },
                match &identification_adresses {
                    Ok(identification_adresses) =>
                        format!("0x{:x}", identification_adresses.role_word),
                    Err(_) => "-".to_string(),
                }
            ),
        ]);
    }
    // Print the table
    print_markdown_table(&table, true);

    // Put group into op state
    match smol::block_on(group.into_op(maindevice)) {
        Ok(_) => println!("Successfully put group into OP state"),
        Err(e) => println!("Failed to put group into OP state: {}", e),
    }
}
