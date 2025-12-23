use std::fmt::Display;

use ethercat_hal::devices::wago_750_354::WAGO_750_354_IDENTITY_A;
use ethercat_hal::devices::wago_modules::ip20_ec_di8_do8::IP20_EC_DI8_DO8_IDENTITY;
use serde::Deserialize;
use serde::Serialize;

/// Identifies a spacifi machine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MachineIdentificationUnique {
    pub machine_identification: MachineIdentification,
    pub serial: u16,
}

impl MachineIdentificationUnique {
    /// Check if values are non-zero
    pub const fn is_valid(&self) -> bool {
        self.machine_identification.is_valid() && self.serial != 0
    }
}

impl Display for MachineIdentificationUnique {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.machine_identification.vendor, self.machine_identification.machine, self.serial
        )
    }
}

/// Identifies a machine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MachineIdentification {
    pub vendor: u16,
    pub machine: u16,
}

impl MachineIdentification {
    /// Check if values are non-zero
    pub const fn is_valid(&self) -> bool {
        self.vendor != 0 && self.machine != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceMachineIdentification {
    pub machine_identification_unique: MachineIdentificationUnique,
    pub role: u16,
}

impl DeviceMachineIdentification {
    /// Check if values are non-zero
    pub const fn is_valid(&self) -> bool {
        self.machine_identification_unique.is_valid()
            && self
                .machine_identification_unique
                .machine_identification
                .machine
                != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceIdentification {
    pub device_machine_identification: Option<DeviceMachineIdentification>,
    pub device_hardware_identification: DeviceHardwareIdentification,
}

impl From<DeviceIdentificationIdentified> for DeviceIdentification {
    fn from(value: DeviceIdentificationIdentified) -> Self {
        Self {
            device_machine_identification: Some(value.device_machine_identification),
            device_hardware_identification: value.device_hardware_identification,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceIdentificationIdentified {
    pub device_machine_identification: DeviceMachineIdentification,
    pub device_hardware_identification: DeviceHardwareIdentification,
}

impl TryFrom<DeviceIdentification> for DeviceIdentificationIdentified {
    type Error = anyhow::Error;

    fn try_from(value: DeviceIdentification) -> Result<Self, Self::Error> {
        let device_machine_identification =
            value.device_machine_identification.ok_or(anyhow::anyhow!(
                "[{}::DeviceIdentificationIdentified::try_from] No device machine identification",
                module_path!()
            ))?;

        Ok(Self {
            device_machine_identification,
            device_hardware_identification: value.device_hardware_identification,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceHardwareIdentification {
    Ethercat(DeviceHardwareIdentificationEthercat),
    Serial(DeviceHardwareIdentificationSerial),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceHardwareIdentificationEthercat {
    pub subdevice_index: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceHardwareIdentificationSerial {
    pub path: String,
}

use anyhow::Error;
use anyhow::anyhow;
use ethercat_hal::devices::ek1100::EK1100_IDENTITY_A;
use ethercat_hal::devices::el1002::EL1002_IDENTITY_A;
use ethercat_hal::devices::el1008::EL1008_IDENTITY_A;
use ethercat_hal::devices::el2002::EL2002_IDENTITY_A;
use ethercat_hal::devices::el2002::EL2002_IDENTITY_B;
use ethercat_hal::devices::el2004::EL2004_IDENTITY_A;
use ethercat_hal::devices::el2008::EL2008_IDENTITY_A;
use ethercat_hal::devices::el2008::EL2008_IDENTITY_B;
use ethercat_hal::devices::el2521::{
    EL2521_IDENTITY_0000_A, EL2521_IDENTITY_0000_B, EL2521_IDENTITY_0024_A,
};
use ethercat_hal::devices::el2522::EL2522_IDENTITY_A;
use ethercat_hal::devices::el3001::EL3001_IDENTITY_A;
use ethercat_hal::devices::el3021::EL3021_IDENTITY_A;
use ethercat_hal::devices::el3024::EL3024_IDENTITY_A;
use ethercat_hal::devices::el3062_0030::EL3062_0030_IDENTITY_A;
use ethercat_hal::devices::el3204::EL3204_IDENTITY_A;
use ethercat_hal::devices::el3204::EL3204_IDENTITY_B;
use ethercat_hal::devices::el4002::EL4002_IDENTITY_A;
use ethercat_hal::devices::el5152::EL5152_IDENTITY_A;
use ethercat_hal::devices::el6021::{
    EL6021_IDENTITY_A, EL6021_IDENTITY_B, EL6021_IDENTITY_C, EL6021_IDENTITY_D,
};
use ethercat_hal::devices::el7031::{EL7031_IDENTITY_A, EL7031_IDENTITY_B};
use ethercat_hal::devices::el7031_0030::EL7031_0030_IDENTITY_A;
use ethercat_hal::devices::el7041_0052::EL7041_0052_IDENTITY_A;
use ethercat_hal::devices::subdevice_identity_to_tuple;
use ethercat_hal::helpers::ethercrab_types::{
    EthercrabSubDeviceOperational, EthercrabSubDevicePreoperational,
};
use ethercrab::MainDevice;
use ethercrab::SubDeviceIdentity;

#[derive(Debug)]
pub struct MachineIdentificationAddresses {
    pub vendor_word: u16,
    pub serial_word: u16,
    pub machine_word: u16,
    pub role_word: u16,
}

impl MachineIdentificationAddresses {
    pub const fn new(
        vendor_word: u16,
        serial_word: u16,
        machine_word: u16,
        device_word: u16,
    ) -> Self {
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
            machine_word: 0x0029,
            serial_word: 0x002a,
            role_word: 0x002b,
        }
    }
}

/// Reads the EEPROM of all subdevices to get their machine device identifications
///
/// Returns a vector of MachineDeviceIdentification for all subdevices
pub async fn read_device_identifications<'maindevice>(
    subdevices: &Vec<EthercrabSubDevicePreoperational<'maindevice>>,
    maindevice: &MainDevice<'maindevice>,
) -> Vec<Result<DeviceMachineIdentification, Error>> {
    let mut result = Vec::new();
    for subdevice in subdevices.iter() {
        let identification = machine_device_identification(subdevice, maindevice).await;
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
            // u16dump(subdevice, maindevice, 0, 128).await?;
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

    tracing::debug!(
        "Read MDI from EEPROM for device {}\nVendor:  0x{:08x} at 0x{:04x}-0x{:04x}\nSerial:  0x{:08x} at 0x{:04x}-0x{:04x}\nMachine: 0x{:08x} at 0x{:04x}-0x{:04x}\nRole:    0x{:08x} at 0x{:04x}-0x{:04x}",
        subdevice.name(),
        mdi.machine_identification_unique
            .machine_identification
            .vendor,
        addresses.vendor_word,
        addresses.vendor_word + 1,
        mdi.machine_identification_unique.serial,
        addresses.serial_word,
        addresses.serial_word + 1,
        mdi.machine_identification_unique
            .machine_identification
            .machine,
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
    tracing::debug!(
        "Writing MDI to EEPROM for device {}\nVendor:  0x{:08x} at 0x{:04x}-0x{:04x}\nSerial:  0x{:08x} at 0x{:04x}-0x{:04x}\nMachine: 0x{:08x} at 0x{:04x}-0x{:04x}\nRole:    0x{:08x} at 0x{:04x}-0x{:04x}",
        subdevice.name(),
        device_identification
            .machine_identification_unique
            .machine_identification
            .vendor,
        addresses.vendor_word,
        addresses.vendor_word + 1,
        device_identification.machine_identification_unique.serial,
        addresses.serial_word,
        addresses.serial_word + 1,
        device_identification
            .machine_identification_unique
            .machine_identification
            .machine,
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
            device_identification
                .machine_identification_unique
                .machine_identification
                .vendor,
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
            device_identification
                .machine_identification_unique
                .machine_identification
                .machine,
        )
        .await?;
    subdevice
        .eeprom_write_dangerously(maindevice, addresses.role_word, device_identification.role)
        .await?;
    Ok(())
}

/// Returns the EEPROM addresses for the machine device identification
/// based on the subdevice's identity
pub fn get_identification_addresses(
    subdevice_identity: &SubDeviceIdentity,
    subdevice_name: &str,
) -> Result<MachineIdentificationAddresses, Error> {
    let identity_tuple = subdevice_identity_to_tuple(subdevice_identity);

    Ok(match identity_tuple {
        WAGO_750_354_IDENTITY_A => MachineIdentificationAddresses::default(),
        IP20_EC_DI8_DO8_IDENTITY => MachineIdentificationAddresses::default(),
        EK1100_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL1002_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL1008_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL2002_IDENTITY_A | EL2002_IDENTITY_B => MachineIdentificationAddresses::default(),
        EL2004_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL2008_IDENTITY_A | EL2008_IDENTITY_B => MachineIdentificationAddresses::default(),
        EL2521_IDENTITY_0000_A | EL2521_IDENTITY_0000_B | EL2521_IDENTITY_0024_A => {
            MachineIdentificationAddresses::default()
        }
        EL2522_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL3001_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL3021_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL3024_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL3062_0030_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL3204_IDENTITY_A | EL3204_IDENTITY_B => MachineIdentificationAddresses::default(),
        EL4002_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL5152_IDENTITY_A => MachineIdentificationAddresses::default(),
        EL6021_IDENTITY_A | EL6021_IDENTITY_B | EL6021_IDENTITY_C | EL6021_IDENTITY_D => {
            MachineIdentificationAddresses::default()
        }
        EL7031_IDENTITY_A | EL7031_IDENTITY_B => MachineIdentificationAddresses::default(),
        EL7031_0030_IDENTITY_A => MachineIdentificationAddresses::default(),
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

#[cfg(test)]
mod tests {
    use super::*;
}
