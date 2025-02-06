use anyhow::anyhow;
use anyhow::Error;
use futures::executor::block_on;

use super::hexdump::u16dump;
use super::types::EthercrabSubDevice;

#[derive(Debug, PartialEq, Default)]
pub struct DeviceGroup {
    pub vendor: u32,
    pub serial: u32,
    pub machine: u32,
}

#[derive(Debug)]
pub struct DeviceGroupDevice {
    pub machine_identification: DeviceGroup,
    pub devices: Vec<(usize, u32)>,
}

/// reads the EEPROM of all subdevices and groups them by machine identification
///
/// Return 0: Vec<DeviceGroupDevice> - a vector of devices grouped by machine identification
/// Return 1: Vec<(usize, MachineDeviceIdentification)> - a vector of devices that could not be identified
pub async fn group_devices(
    subdevices: &[EthercrabSubDevice<'_>],
) -> Result<
    (
        Vec<DeviceGroupDevice>,
        Vec<(usize, MachineDeviceIdentification)>,
    ),
    Error,
> {
    let mut device_groups: Vec<DeviceGroupDevice> = Vec::new();
    let mut unidentified_devices: Vec<(usize, MachineDeviceIdentification)> = Vec::new();

    for (i, subdevice) in subdevices.iter().enumerate() {
        let mdid = machine_device_identification(subdevice).await?;

        // if vendor or serial or machine is 0, it is not a valid machine device
        if mdid.machine_identification == DeviceGroup::default() {
            unidentified_devices.push((i, mdid));
            continue;
        }

        let mut found = false;
        for machine in device_groups.iter_mut() {
            if machine.machine_identification == mdid.machine_identification {
                machine.devices.push((i, mdid.device));
                found = true;
                break;
            }
        }
        if !found {
            device_groups.push(DeviceGroupDevice {
                machine_identification: mdid.machine_identification,
                devices: vec![(i, mdid.device)],
            });
        }
    }

    Ok((device_groups, unidentified_devices))
}

#[derive(Debug)]
pub struct MachineDeviceIdentification {
    machine_identification: DeviceGroup,
    device: u32,
}

/// Reads the machine device identification from the EEPROM
pub async fn machine_device_identification(
    subdevice: &EthercrabSubDevice<'_>,
) -> Result<MachineDeviceIdentification, Error> {
    let eeprom = subdevice.eeprom();
    let addresses = get_identification_addresses(subdevice)?;
    Ok(MachineDeviceIdentification {
        machine_identification: DeviceGroup {
            vendor: words_to_u32be(
                eeprom.read(addresses.vendor_word).await.unwrap(),
                eeprom.read(addresses.vendor_word + 1).await.unwrap(),
            ),
            serial: words_to_u32be(
                eeprom.read(addresses.serial_word).await.unwrap(),
                eeprom.read(addresses.serial_word + 1).await.unwrap(),
            ),
            machine: words_to_u32be(
                eeprom.read(addresses.machine_word).await.unwrap(),
                eeprom.read(addresses.machine_word + 1).await.unwrap(),
            ),
        },
        device: words_to_u32be(
            eeprom.read(addresses.device_word).await.unwrap(),
            eeprom.read(addresses.device_word + 1).await.unwrap(),
        ),
    })
}

/// Writes the machine device identification to the EEPROM
pub async fn write_machine_device_identification(
    subdevice: &EthercrabSubDevice<'_>,
    identification: MachineDeviceIdentification,
) -> Result<(), Error> {
    let eeprom = subdevice.eeprom();
    let addresses = get_identification_addresses(subdevice)?;
    eeprom
        .write(
            addresses.vendor_word,
            identification.machine_identification.vendor as u16,
        )
        .await?;
    eeprom
        .write(
            addresses.vendor_word + 1,
            (identification.machine_identification.vendor >> 16) as u16,
        )
        .await?;
    eeprom
        .write(
            addresses.serial_word,
            identification.machine_identification.serial as u16,
        )
        .await?;
    eeprom
        .write(
            addresses.serial_word + 1,
            (identification.machine_identification.serial >> 16) as u16,
        )
        .await?;
    eeprom
        .write(
            addresses.machine_word,
            identification.machine_identification.machine as u16,
        )
        .await?;
    eeprom
        .write(
            addresses.machine_word + 1,
            (identification.machine_identification.machine >> 16) as u16,
        )
        .await?;
    eeprom
        .write(addresses.device_word, identification.device as u16)
        .await?;
    eeprom
        .write(
            addresses.device_word + 1,
            (identification.device >> 16) as u16,
        )
        .await?;
    Ok(())
}

/// Converts two u16 words to a u32 big endian
fn words_to_u32be(word_low: u16, word_high: u16) -> u32 {
    ((word_high as u32) << 16) | word_low as u32
}

#[derive(Debug)]
pub struct MachineDeviceIdentificationAddresses {
    vendor_word: u16,
    serial_word: u16,
    machine_word: u16,
    device_word: u16,
}

impl MachineDeviceIdentificationAddresses {
    pub fn new(vendor_word: u16, serial_word: u16, machine_word: u16, device_word: u16) -> Self {
        Self {
            vendor_word,
            serial_word,
            machine_word,
            device_word,
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
            device_word: 0x002e,
        }
    }
}

/// Returns the EEPROM addresses for the machine device identification
/// based on the subdevice's identity
pub fn get_identification_addresses(
    subdevice: &EthercrabSubDevice,
) -> Result<MachineDeviceIdentificationAddresses, Error> {
    let identity = subdevice.identity();
    let identity_tuple = (identity.vendor_id, identity.product_id, identity.revision);

    Ok(match identity_tuple {
        (BECKHOFF, EK1100, 0x00120000) => MachineDeviceIdentificationAddresses::default(),
        (BECKHOFF, EL1008, 0x00110000) => MachineDeviceIdentificationAddresses::default(),
        (BECKHOFF, EL2008, 0x00110000) => MachineDeviceIdentificationAddresses::default(),
        (BECKHOFF, EL4008, 0x00140000) => MachineDeviceIdentificationAddresses::default(),
        (BECKHOFF, EL3204, 0x00150000) => MachineDeviceIdentificationAddresses::default(),
        _ => {
            block_on(u16dump(&subdevice, 0x00, 0xff))?;
            Err(anyhow!(
            "Unknown MDI addresses for device {:?} vendor: 0x{:08x} product: 0x{:08x} revision: 0x{:08x}",
            subdevice.name(),
            identity.vendor_id,
            identity.product_id,
            identity.revision
        ))?
        }
    })
}

// === VENDOR IDS ===
const BECKHOFF: u32 = 0x00000002;

// === PRODUCTS ===
const EK1100: u32 = 0x044c2c52;
const EL1008: u32 = 0x03f03052;
const EL2008: u32 = 0x07d83052;
const EL4008: u32 = 0x0fa83052;
const EL3204: u32 = 0x0c843052;
