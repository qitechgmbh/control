use std::fmt::Display;

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
use ethercat_hal::devices::el7031::EL7031_IDENTITY_A;
use ethercat_hal::devices::el7041_0052::EL7041_0052_IDENTITY_A;
use ethercat_hal::devices::subdevice_identity_to_tuple;
use ethercat_hal::types::EthercrabSubDeviceGroupPreoperational;
use ethercat_hal::types::EthercrabSubDeviceOperational;
use ethercat_hal::types::EthercrabSubDevicePreoperational;
use ethercrab::MainDevice;
use ethercrab::SubDeviceIdentity;
use serde::Deserialize;
use serde::Serialize;

/// Identifies a spacifi machine
#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize, Eq, Hash)]
pub struct MachineIdentificationUnique {
    pub vendor: u32,
    pub serial: u32,
    pub machine: u32,
}

impl Display for MachineIdentificationUnique {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "vendor: {}, serial: {}, machine: {}",
            self.vendor, self.serial, self.machine
        )
    }
}

/// Identifies a machine
#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct MachineIdentification {
    pub vendor: u32,
    pub machine: u32,
}

impl MachineIdentification {
    pub fn new(vendor: u32, machine: u32) -> Self {
        Self { vendor, machine }
    }
}

impl Display for MachineIdentification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "vendor: {}, machine: {}", self.vendor, self.machine)
    }
}

impl From<&MachineIdentificationUnique> for MachineIdentification {
    fn from(mdi: &MachineIdentificationUnique) -> Self {
        Self {
            vendor: mdi.vendor,
            machine: mdi.machine,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineDeviceIdentification {
    pub machine_identification_unique: MachineIdentificationUnique,
    pub role: u32,
    pub subdevice_index: usize,
}

impl Display for MachineDeviceIdentification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} role: 0x{:08x} subdevice_index: {}",
            self.machine_identification_unique, self.role, self.subdevice_index
        )
    }
}

#[derive(Debug)]
pub struct MachineDeviceIdentificationAddresses {
    pub vendor_word: u16,
    pub serial_word: u16,
    pub machine_word: u16,
    pub role_word: u16,
}

impl MachineDeviceIdentificationAddresses {
    pub fn new(vendor_word: u16, serial_word: u16, machine_word: u16, device_word: u16) -> Self {
        Self {
            vendor_word,
            serial_word,
            machine_word,
            role_word: device_word,
        }
    }
}

impl Default for MachineDeviceIdentificationAddresses {
    fn default() -> Self {
        Self {
            // 0x0028 to 0x0029 BE
            vendor_word: 0x0028,
            // 0x002a to 0x002b BE
            serial_word: 0x002a,
            // 0x002c to 0x002d BE
            machine_word: 0x002c,
            // 0x002e to 0x002f BE
            role_word: 0x002e,
        }
    }
}

/// reads the EEPROM of all subdevices and groups them by machine identification
///
/// Return 0: Vec<DeviceGroupDevice> - a vector of devices grouped by machine identification
/// Return 1: Vec<(usize, MachineDeviceIdentification)> - a vector of devices that could not be identified
pub async fn identify_device_groups<
    'maindevice,
    const MAX_SUBDEVICES: usize,
    const MAX_PDI: usize,
>(
    subdevices: &EthercrabSubDeviceGroupPreoperational<MAX_SUBDEVICES, MAX_PDI>,
    maindevice: &MainDevice<'maindevice>,
) -> Result<
    (
        Vec<Vec<MachineDeviceIdentification>>,
        Vec<MachineDeviceIdentification>,
    ),
    Error,
> {
    let mut device_groups: Vec<Vec<MachineDeviceIdentification>> = Vec::new();
    // 0: subdevice index 1: machine device identification
    let mut unidentified_devices: Vec<MachineDeviceIdentification> = Vec::new();

    for (subdevice_index, subdevice) in subdevices.iter(maindevice).enumerate() {
        let mdid = machine_device_identification(&subdevice, subdevice_index, maindevice).await?;

        // if vendor or serial or machine is 0, it is not a valid machine device
        if mdid.machine_identification_unique == MachineIdentificationUnique::default() {
            unidentified_devices.push(mdid);
            continue;
        }

        let mut found = false;
        for device_group in device_groups.iter_mut() {
            if device_group
                .first()
                .map(|d| &d.machine_identification_unique)
                == Some(&mdid.machine_identification_unique)
            {
                device_group.push(mdid.clone());
                found = true;
                break;
            }
        }
        if !found {
            device_groups.push(vec![mdid]);
        }
    }

    Ok((device_groups, unidentified_devices))
}

/// Reads the machine device identification from the EEPROM
pub async fn machine_device_identification<'maindevice>(
    subdevice: &'maindevice EthercrabSubDevicePreoperational<'maindevice>,
    subdevice_index: usize,
    maindevice: &MainDevice<'_>,
) -> Result<MachineDeviceIdentification, Error> {
    let addresses = match get_identification_addresses(&subdevice.identity(), subdevice.name()) {
        Ok(x) => x,
        Err(e) => {
            u16dump(subdevice, maindevice, 0, 128).await?;
            return Err(e);
        }
    };

    Ok(MachineDeviceIdentification {
        machine_identification_unique: MachineIdentificationUnique {
            vendor: words_to_u32be(
                subdevice
                    .eeprom_read(maindevice, addresses.vendor_word)
                    .await
                    .unwrap(),
                subdevice
                    .eeprom_read(maindevice, addresses.vendor_word + 1)
                    .await
                    .unwrap(),
            ),
            serial: words_to_u32be(
                subdevice
                    .eeprom_read(maindevice, addresses.serial_word)
                    .await
                    .unwrap(),
                subdevice
                    .eeprom_read(maindevice, addresses.serial_word + 1)
                    .await
                    .unwrap(),
            ),
            machine: words_to_u32be(
                subdevice
                    .eeprom_read(maindevice, addresses.machine_word)
                    .await
                    .unwrap(),
                subdevice
                    .eeprom_read(maindevice, addresses.machine_word + 1)
                    .await
                    .unwrap(),
            ),
        },
        role: words_to_u32be(
            subdevice
                .eeprom_read(maindevice, addresses.role_word)
                .await
                .unwrap(),
            subdevice
                .eeprom_read(maindevice, addresses.role_word + 1)
                .await
                .unwrap(),
        ),
        subdevice_index: subdevice_index,
    })
}

/// Writes the machine device identification to the EEPROM
pub async fn write_machine_device_identification<'maindevice, const MAX_PDI: usize>(
    subdevice: &EthercrabSubDeviceOperational<'maindevice, MAX_PDI>,
    maindevice: &MainDevice<'_>,
    identification: &MachineDeviceIdentification,
) -> Result<(), Error> {
    let addresses = get_identification_addresses(&subdevice.identity(), subdevice.name())?;

    subdevice
        .eeprom_write_dangerously(
            maindevice,
            addresses.vendor_word,
            identification.machine_identification_unique.vendor as u16,
        )
        .await?;
    subdevice
        .eeprom_write_dangerously(
            maindevice,
            addresses.vendor_word + 1,
            (identification.machine_identification_unique.vendor >> 16) as u16,
        )
        .await?;
    subdevice
        .eeprom_write_dangerously(
            maindevice,
            addresses.serial_word,
            identification.machine_identification_unique.serial as u16,
        )
        .await?;
    subdevice
        .eeprom_write_dangerously(
            maindevice,
            addresses.serial_word + 1,
            (identification.machine_identification_unique.serial >> 16) as u16,
        )
        .await?;
    subdevice
        .eeprom_write_dangerously(
            maindevice,
            addresses.machine_word,
            identification.machine_identification_unique.machine as u16,
        )
        .await?;
    subdevice
        .eeprom_write_dangerously(
            maindevice,
            addresses.machine_word + 1,
            (identification.machine_identification_unique.machine >> 16) as u16,
        )
        .await?;
    subdevice
        .eeprom_write_dangerously(maindevice, addresses.role_word, identification.role as u16)
        .await?;
    subdevice
        .eeprom_write_dangerously(
            maindevice,
            addresses.role_word + 1,
            (identification.role >> 16) as u16,
        )
        .await?;
    Ok(())
}

/// Converts two u16 words to a u32 big endian
fn words_to_u32be(word_low: u16, word_high: u16) -> u32 {
    ((word_high as u32) << 16) | word_low as u32
}

/// Returns the EEPROM addresses for the machine device identification
/// based on the subdevice's identity
pub fn get_identification_addresses<'maindevice>(
    subdevice_identity: &SubDeviceIdentity,
    subdevice_name: &str,
) -> Result<MachineDeviceIdentificationAddresses, Error> {
    let identity_tuple = subdevice_identity_to_tuple(&subdevice_identity);

    Ok(match identity_tuple {
        EK1100_IDENTITY_A => MachineDeviceIdentificationAddresses::default(),
        EL1008_IDENTITY_A => MachineDeviceIdentificationAddresses::default(),
        EL2002_IDENTITY_A => MachineDeviceIdentificationAddresses::default(),
        EL2008_IDENTITY_A => MachineDeviceIdentificationAddresses::default(),
        EL3001_IDENTITY_A => MachineDeviceIdentificationAddresses::default(),
        EL2521_IDENTITY_0000_A | EL2521_IDENTITY_0000_B | EL2521_IDENTITY_0024_A => {
            MachineDeviceIdentificationAddresses::default()
        }
        EL2522_IDENTITY_A => MachineDeviceIdentificationAddresses::default(),
        EL3024_IDENTITY_A => MachineDeviceIdentificationAddresses::default(),
        EL3021_IDENTITY_A => MachineDeviceIdentificationAddresses::default(),
        EL7031_IDENTITY_A => MachineDeviceIdentificationAddresses::default(),
        EL7041_0052_IDENTITY_A => MachineDeviceIdentificationAddresses::default(),
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
