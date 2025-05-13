use anyhow::Error;
use anyhow::anyhow;
use ethercat_hal::devices::ek1100::EK1100_IDENTITY_A;
use ethercat_hal::devices::el1008::EL1008_IDENTITY_A;
use ethercat_hal::devices::el2002::EL2002_IDENTITY_A;
use ethercat_hal::devices::el2008::EL2008_IDENTITY_A;
use ethercat_hal::devices::el2521::{
    EL2521_IDENTITY_0000_A, EL2521_IDENTITY_0000_B, EL2521_IDENTITY_0024_A,
};
use ethercat_hal::devices::el2522::EL2522_IDENTITY_A;
use ethercat_hal::devices::el3001::EL3001_IDENTITY_A;
use ethercat_hal::devices::el3021::EL3021_IDENTITY_A;
use ethercat_hal::devices::el3024::EL3024_IDENTITY_A;
use ethercat_hal::devices::el3204::EL3204_IDENTITY_A;
use ethercat_hal::devices::el3204::EL3204_IDENTITY_B;

use ethercat_hal::devices::el7031::EL7031_IDENTITY_A;
use ethercat_hal::devices::el7031::EL7031_IDENTITY_B;
use ethercat_hal::devices::el7041_0052::EL7041_0052_IDENTITY_A;
use ethercat_hal::devices::subdevice_identity_to_tuple;
use ethercat_hal::types::{
    EthercrabSubDeviceOperational,
    EthercrabSubDevicePreoperational,
};
use ethercrab::MainDevice;
use ethercrab::SubDeviceIdentity;

use crate::machines::identification::DeviceMachineIdentification;
use crate::machines::identification::MachineIdentification;
use crate::machines::identification::MachineIdentificationUnique;

#[derive(Debug)]
pub struct MachineIdentificationAddresses {
    pub vendor_word: u16,
    pub serial_word: u16,
    pub machine_word: u16,
    pub role_word: u16,
}

impl MachineIdentificationAddresses {
    pub fn new(vendor_word: u16, serial_word: u16, machine_word: u16, device_word: u16) -> Self {
        Self {
            vendor_word,
            serial_word,
            machine_word,
            role_word: device_word,
        }
    }
}

impl Default for MachineIdentificationAddresses {
    fn default() -> Self {
        Self {
            vendor_word: 0x0028,
            serial_word: 0x0029,
            machine_word: 0x002a,
            role_word: 0x002b,
        }
    }
}

/// Reads the EEPROM of all subdevices to get their machine device identifications
///
/// Returns a vector of MachineDeviceIdentification for all subdevices
pub async fn read_device_identifications<
    'maindevice,
>(
    subdevices: &Vec<EthercrabSubDevicePreoperational<'maindevice>>,
    maindevice: &MainDevice<'maindevice>,
) -> Vec<Result<DeviceMachineIdentification, Error>> {
    let mut result = Vec::new();
    for subdevice in subdevices.iter() {
        let identification = machine_device_identification(&subdevice, maindevice).await;
        result.push(identification);
    }
    result
}

/// Reads the machine device identification from the EEPROM
pub async fn machine_device_identification<'maindevice>(
    subdevice: &'maindevice EthercrabSubDevicePreoperational<'maindevice>,
    maindevice: &MainDevice<'_>,
) -> Result<DeviceMachineIdentification, Error> {
    let addresses = match get_identification_addresses(&subdevice.identity(), subdevice.name()) {
        Ok(x) => x,
        Err(e) => {
            u16dump(subdevice, maindevice, 0, 128).await?;
            return Err(e);
        }
    };

    let mdi = DeviceMachineIdentification {
        machine_identification_unique: MachineIdentificationUnique {
            machine_identification: MachineIdentification {
                vendor: subdevice
                .eeprom_read::<u16>(maindevice, addresses.vendor_word)
                .await
                .or(Err(anyhow!(
                    "[{}::machine_device_identification] Failed to read vendor from EEPROM for device {}",
                    module_path!(),
                    subdevice.name()
                )))?,
                machine: subdevice
                .eeprom_read::<u16>(maindevice, addresses.machine_word)
                .await
                .or(Err(anyhow!(
                    "[{}::machine_device_identification] Failed to read machine from EEPROM for device {}",
                    module_path!(),
                    subdevice.name()
                )))?,
            },
            serial: subdevice
                .eeprom_read::<u16>(maindevice, addresses.serial_word)
                .await
                .or(Err(anyhow!(
                    "[{}::machine_device_identification] Failed to read serial from EEPROM for device {}",
                    module_path!(),
                    subdevice.name()
                )))?,
           
        },
        role: subdevice
            .eeprom_read::<u16>(maindevice, addresses.role_word)
            .await
            .or(Err(anyhow!(
                "[{}::machine_device_identification] Failed to read role from EEPROM for device {}",
                module_path!(),
                subdevice.name()
            )))?,
    };

    log::debug!(
        "[{}::machine_device_identification] Read MDI from EEPROM for device {}\nVendor:  0x{:08x} at 0x{:04x}-0x{:04x}\nSerial:  0x{:08x} at 0x{:04x}-0x{:04x}\nMachine: 0x{:08x} at 0x{:04x}-0x{:04x}\nRole:    0x{:08x} at 0x{:04x}-0x{:04x}",
        module_path!(),
        subdevice.name(),
        mdi.machine_identification_unique.machine_identification.vendor,
        addresses.vendor_word,
        addresses.vendor_word + 1,
        mdi.machine_identification_unique.serial,
        addresses.serial_word,
        addresses.serial_word + 1,
        mdi.machine_identification_unique.machine_identification.machine,
        addresses.machine_word,
        addresses.machine_word + 1,
        mdi.role,
        addresses.role_word,
        addresses.role_word + 1,
    );

    Ok(mdi)
}

/// Writes the machine device identification to the EEPROM
pub async fn write_machine_device_identification<'maindevice, const MAX_PDI: usize>(
    subdevice: &EthercrabSubDeviceOperational<'maindevice, MAX_PDI>,
    maindevice: &MainDevice<'_>,
    device_identification: &DeviceMachineIdentification,
) -> Result<(), Error> {
    let addresses = get_identification_addresses(&subdevice.identity(), subdevice.name())?;
    log::debug!(
        "[{}::write_machine_device_identification] Writing MDI to EEPROM for device {}\nVendor:  0x{:08x} at 0x{:04x}-0x{:04x}\nSerial:  0x{:08x} at 0x{:04x}-0x{:04x}\nMachine: 0x{:08x} at 0x{:04x}-0x{:04x}\nRole:    0x{:08x} at 0x{:04x}-0x{:04x}",
        module_path!(),
        subdevice.name(),
        device_identification.machine_identification_unique.machine_identification.vendor,
        addresses.vendor_word,
        addresses.vendor_word + 1,
        device_identification.machine_identification_unique.serial,
        addresses.serial_word,
        addresses.serial_word + 1,
        device_identification.machine_identification_unique.machine_identification.machine,
        addresses.machine_word,
        addresses.machine_word + 1,
        device_identification.role,
        addresses.role_word,
        addresses.role_word + 1,
    );

    subdevice
        .eeprom_write_dangerously(
            maindevice,
            addresses.vendor_word,
            device_identification.machine_identification_unique.machine_identification.vendor,
        )
        .await?;
    subdevice
        .eeprom_write_dangerously(
            maindevice,
            addresses.serial_word,
            device_identification.machine_identification_unique.serial,
        )
        .await?;
    subdevice
        .eeprom_write_dangerously(
            maindevice,
            addresses.machine_word,
            device_identification.machine_identification_unique.machine_identification.machine,
        )
        .await?;
    subdevice
        .eeprom_write_dangerously(maindevice, addresses.role_word, device_identification.role)
        .await?;
    Ok(())
}

/// Returns the EEPROM addresses for the machine device identification
/// based on the subdevice's identity
pub fn get_identification_addresses<'maindevice>(
    subdevice_identity: &SubDeviceIdentity,
    subdevice_name: &str,
) -> Result<MachineIdentificationAddresses, Error> {
    let identity_tuple = subdevice_identity_to_tuple(&subdevice_identity);

    Ok(match identity_tuple {
        EK1100_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL1008_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL2002_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL3204_IDENTITY_A | EL3204_IDENTITY_B => MachineIdentificationAddresses::default(),
        EL2008_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL3001_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL2521_IDENTITY_0000_A | EL2521_IDENTITY_0000_B | EL2521_IDENTITY_0024_A => {
            MachineIdentificationAddresses::default()
        }
        EL2522_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL3024_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL3021_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL7031_IDENTITY_A | EL7031_IDENTITY_B => MachineIdentificationAddresses::default(),
        EL7041_0052_IDENTITY_A => MachineIdentificationAddresses::default(),
        _ => {
            // block_on(u16dump(&subdevice, maindevice, 0x00, 0xff))?;
            Err(anyhow!(
                "[{}::get_identification_addresses] Unknown MDI addresses for device {:?} vendor: 0x{:08x} product: 0x{:08x} revision: 0x{:08x}",
                module_path!(),
                subdevice_name,
                subdevice_identity.vendor_id,
                subdevice_identity.product_id,
                subdevice_identity.revision
            ))?
        }
    })
}

async fn u16dump<'maindevice>(
    subdevice: &'maindevice EthercrabSubDevicePreoperational<'maindevice>,
    maindevice: &MainDevice<'maindevice>,
    start_byte: u16,
    end_byte: u16,
) -> Result<(), Error> {
    let mut words: Vec<u16> = Vec::new();
    for word in start_byte..end_byte {
        words.push(subdevice.eeprom_read(maindevice, word).await?);
    }

    print!(
        "EEPROM dump for {} from 0x{:04x} to 0x{:04x}\n",
        subdevice.name(),
        start_byte / 2,
        end_byte / 2
    );

    u16print(start_byte, end_byte, words);

    Ok(())
}

fn u16print(start_byte: u16, end_byte: u16, data: Vec<u16>) {
    let table_start_word = start_byte & 0xfff0;
    let table_end_word = (end_byte & 0xfff0_u16) + 0x10_u16;

    let rows = table_end_word - table_start_word >> 4;

    for row in 0..rows {
        print!("0x{:04x} | ", (table_start_word + row * 0x10) / 2);
        for word in 0..8 {
            let word_address = row * 8 + word;
            if word_address < start_byte {
                print!("     ");
            } else {
                let i = (word_address - start_byte) as usize;
                if i > data.len() - 1 {
                    print!("     ");
                } else {
                    print!("{:04x} ", data[i]);
                }
            }
        }
        print!("\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hexprint() {
        let data = vec![0x0000, 0x1ced];
        u16print(0x01, 0x40, data);
    }
}
