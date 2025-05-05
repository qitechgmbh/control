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
    pub fn is_valid(&self) -> bool {
        self.machine_identification.is_valid() && self.serial != 0
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
    pub fn is_valid(&self) -> bool {
        self.vendor != 0 && self.machine != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceMachineIdentification {
    pub machine_identification_unique: MachineIdentificationUnique,
    pub role: u16,
}

impl DeviceMachineIdentification {
    /// Check if values are non-zero
    pub fn is_valid(&self) -> bool {
        self.machine_identification_unique.is_valid()
            && self
                .machine_identification_unique
                .machine_identification
                .machine
                != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceIdentification {
    pub device_machine_identification: Option<DeviceMachineIdentification>,
    pub device_hardware_identification: DeviceHardwareIdentification,
}

impl From<DeviceIdentificationIdentified> for DeviceIdentification {
    fn from(value: DeviceIdentificationIdentified) -> Self {
        DeviceIdentification {
            device_machine_identification: Some(value.device_machine_identification),
            device_hardware_identification: value.device_hardware_identification,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

        Ok(DeviceIdentificationIdentified {
            device_machine_identification,
            device_hardware_identification: value.device_hardware_identification,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceHardwareIdentification {
    Ethercat(DeviceHardwareIdentificationEthercat),
    // UsbDevice...,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceHardwareIdentificationEthercat {
    pub subdevice_index: usize,
}
