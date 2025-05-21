use crate::serial::SerialDevice;

use super::identification::DeviceIdentificationIdentified;
use anyhow::Error;
use ethercat_hal::{devices::EthercatDevice, helpers::ethercrab_types::EthercrabSubDevicePreoperational};
use ethercrab::{SubDevice, SubDeviceRef};
use smol::lock::RwLock;
use std::sync::Arc;

pub trait MachineNewTrait {
    fn new<'maindevice, 'subdevices>(
        params: &MachineNewParams<'maindevice, 'subdevices, '_, '_, '_, '_, '_>,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}

pub struct MachineNewParams<
    'maindevice,
    'subdevices,
    'device_identifications_identified,
    'ethercat_devices,
    'machine_new_hardware_etehrcat,
    'machine_new_hardware_serial,
    'machine_new_hardware,
> where
    'maindevice: 'machine_new_hardware,
    'subdevices: 'machine_new_hardware,
    'ethercat_devices: 'machine_new_hardware,
    'machine_new_hardware_etehrcat: 'machine_new_hardware,
{
    pub device_group: &'device_identifications_identified Vec<DeviceIdentificationIdentified>,
    pub hardware: &'machine_new_hardware MachineNewHardware<
        'maindevice,
        'subdevices,
        'ethercat_devices,
        'machine_new_hardware_etehrcat,
        'machine_new_hardware_serial,
    >,
}

pub enum MachineNewHardware<
    'maindevice,
    'subdevices,
    'ethercat_devices,
    'machine_new_hardware_etehrcat,
    'machine_new_hardware_serial,
> where
    'maindevice: 'machine_new_hardware_etehrcat,
    'subdevices: 'machine_new_hardware_etehrcat,
    'ethercat_devices: 'machine_new_hardware_etehrcat,
{
    Ethercat(
        &'machine_new_hardware_etehrcat MachineNewHardwareEthercat<
            'maindevice,
            'subdevices,
            'ethercat_devices,
        >,
    ),
    Serial(&'machine_new_hardware_serial MachineNewHardwareSerial),
}

pub struct MachineNewHardwareEthercat<'maindevice, 'subdevices, 'ethercat_devices> {
    pub subdevices:
        &'subdevices Vec<&'subdevices SubDeviceRef<'maindevice, &'subdevices SubDevice>>,
    pub ethercat_devices: &'ethercat_devices Vec<Arc<RwLock<dyn EthercatDevice>>>,
}

pub struct MachineNewHardwareSerial {
    pub device: Arc<RwLock<dyn SerialDevice>>,
}

// validates that all devices in the group have the same machine identification
pub fn validate_same_machine_identification_unique(
    identified_device_group: &Vec<DeviceIdentificationIdentified>,
) -> Result<(), Error> {
    let machine_identification_unique = &identified_device_group
        .first()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "[{}::validate_same_machine_identification] No devices in group",
                module_path!()
            )
        })?
        .device_machine_identification
        .machine_identification_unique;
    for device in identified_device_group.iter() {
        if device
            .device_machine_identification
            .machine_identification_unique
            != *machine_identification_unique
        {
            return Err(anyhow::anyhow!(
                "[{}::validate_same_machine_identification] Different machine identifications",
                module_path!()
            ));
        }
    }
    Ok(())
}

/// validates that every role is unique
pub fn validate_no_role_dublicates(
    identified_device_group: &Vec<DeviceIdentificationIdentified>,
) -> Result<(), Error> {
    let mut roles = vec![];
    for device in identified_device_group.iter() {
        if roles.contains(&device.device_machine_identification.role) {
            return Err(anyhow::anyhow!(
                "[{}::validate_no_role_dublicates] Role dublicate",
                module_path!(),
            ));
        }
        roles.push(device.device_machine_identification.role);
    }
    Ok(())
}

// Inside control_core::machines::new module:
pub fn get_device_identification_by_role(
    identified_device_group: &Vec<DeviceIdentificationIdentified>,
    role: u16,
) -> Result<&DeviceIdentificationIdentified, Error> {
    for device in identified_device_group.iter() {
        if device.device_machine_identification.role == role {
            return Ok(device);
        }
    }
    Err(anyhow::anyhow!(
        "[{}::get_device_identification_by_role] Role {} not found",
        module_path!(),
        role
    ))
}

pub fn get_device_by_index<'maindevice>(
    devices: &Vec<Arc<RwLock<dyn EthercatDevice>>>,
    subdevice_index: usize,
) -> Result<Arc<RwLock<dyn EthercatDevice>>, Error> {
    Ok(devices
        .get(subdevice_index)
        .ok_or(anyhow::anyhow!(
            "[{}::get_device_by_index] Index {} out of bounds for devices",
            module_path!(),
            subdevice_index
        ))?
        .clone())
}

pub fn get_subdevice_by_index<'subdevices, 'maindevice>(
    subdevices: &'subdevices Vec<&EthercrabSubDevicePreoperational<'maindevice>>,
    subdevice_index: usize,
) -> Result<&'subdevices EthercrabSubDevicePreoperational<'maindevice>, Error> {
    Ok(subdevices.get(subdevice_index).ok_or(anyhow::anyhow!(
        "Index {} out of bounds for subdevices",
        subdevice_index
    ))?)
}

pub fn get_ethercat_device_by_index<'maindevice>(
    ethercat_devices: &Vec<Arc<RwLock<dyn EthercatDevice>>>,
    subdevice_index: usize,
) -> Result<Arc<RwLock<dyn EthercatDevice>>, Error> {
    Ok(ethercat_devices
        .get(subdevice_index)
        .ok_or(anyhow::anyhow!(
            "[{}::get_ethercat_device_by_index] Index {} out of bounds for ethercat devices",
            module_path!(),
            subdevice_index
        ))?
        .clone())
}

#[cfg(test)]
mod tests {
    use crate::machines::identification::{
        DeviceHardwareIdentification, DeviceHardwareIdentificationEthercat,
        DeviceMachineIdentification, MachineIdentification, MachineIdentificationUnique,
    };

    pub use super::*;

    #[test]
    fn test_get_device_identification_by_role() {
        let device_identifications = vec![
            // role 0
            DeviceIdentificationIdentified {
                device_machine_identification: DeviceMachineIdentification {
                    machine_identification_unique: MachineIdentificationUnique {
                        machine_identification: MachineIdentification {
                            vendor: 1,
                            machine: 1,
                        },
                        serial: 1,
                    },
                    role: 0,
                },
                device_hardware_identification: DeviceHardwareIdentification::Ethercat(
                    DeviceHardwareIdentificationEthercat { subdevice_index: 0 },
                ),
            },
            // role 1
            DeviceIdentificationIdentified {
                device_machine_identification: DeviceMachineIdentification {
                    machine_identification_unique: MachineIdentificationUnique {
                        machine_identification: MachineIdentification {
                            vendor: 2,
                            machine: 2,
                        },
                        serial: 2,
                    },
                    role: 2,
                },
                device_hardware_identification: DeviceHardwareIdentification::Ethercat(
                    DeviceHardwareIdentificationEthercat { subdevice_index: 1 },
                ),
            },
            // role 2
            DeviceIdentificationIdentified {
                device_machine_identification: DeviceMachineIdentification {
                    machine_identification_unique: MachineIdentificationUnique {
                        machine_identification: MachineIdentification {
                            vendor: 3,
                            machine: 3,
                        },
                        serial: 3,
                    },
                    role: 1,
                },
                device_hardware_identification: DeviceHardwareIdentification::Ethercat(
                    DeviceHardwareIdentificationEthercat { subdevice_index: 2 },
                ),
            },
        ];

        // search for role 0
        let result = get_device_identification_by_role(&device_identifications, 0);
        assert_eq!(result.unwrap().device_machine_identification.role, 0,);

        // search for role 1
        let result = get_device_identification_by_role(&device_identifications, 1);
        assert_eq!(result.unwrap().device_machine_identification.role, 1,);
        // search for role 2
        let result = get_device_identification_by_role(&device_identifications, 2);
        assert_eq!(result.unwrap().device_machine_identification.role, 2,);
    }
}
