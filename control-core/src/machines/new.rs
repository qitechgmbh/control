use super::Machine;
use crate::identification::MachineDeviceIdentification;
use anyhow::Error;
use ethercat_hal::{devices::Device, types::EthercrabSubDevicePreoperational};
use ethercrab::{SubDevice, SubDeviceRef};
use smol::lock::RwLock;
use std::sync::Arc;

pub trait MachineNewTrait {
    fn new<'maindevice, 'subdevices>(
        identified_device_group: &Vec<MachineDeviceIdentification>,
        subdevices: &'subdevices Vec<SubDeviceRef<'maindevice, &SubDevice>>,
        devices: &Vec<Arc<RwLock<dyn Device>>>,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}

pub type MachineNewFn = Box<
    dyn Fn(
            &Vec<MachineDeviceIdentification>,
            &'_ Vec<SubDeviceRef<'_, &SubDevice>>,
            &Vec<Arc<RwLock<dyn Device>>>,
        ) -> Result<Arc<RwLock<dyn Machine>>, Error>
        + Send
        + Sync,
>;

// validates that all devices in the group have the same machine identification
pub fn validate_same_machine_identification(
    identified_device_group: &Vec<MachineDeviceIdentification>,
) -> Result<(), Error> {
    let machine_identification_unique = &identified_device_group
        .first()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "[{}::validate_same_machine_identification] No devices in group",
                module_path!()
            )
        })?
        .machine_identification_unique;
    for device in identified_device_group.iter() {
        if device.machine_identification_unique != *machine_identification_unique {
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
    identified_device_group: &Vec<MachineDeviceIdentification>,
) -> Result<(), Error> {
    let mut roles = vec![];
    for device in identified_device_group.iter() {
        if roles.contains(&device.role) {
            return Err(anyhow::anyhow!(
                "[{}::validate_no_role_dublicates] Role dublicate",
                module_path!(),
            ));
        }
        roles.push(device.role);
    }
    Ok(())
}

/// get a device with a device group
pub fn get_mdi_by_role(
    identified_device_group: &Vec<MachineDeviceIdentification>,
    role: u16,
) -> Result<&MachineDeviceIdentification, Error> {
    for device in identified_device_group.iter() {
        if device.role == role {
            return Ok(device);
        }
    }
    Err(anyhow::anyhow!(
        "[{}::get_mdi_by_role] Device not found",
        module_path!(),
    ))
}

pub fn get_device_by_index<'maindevice>(
    devices: &Vec<Arc<RwLock<dyn Device>>>,
    subdevice_index: usize,
) -> Result<Arc<RwLock<dyn Device>>, Error> {
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
    subdevices: &'subdevices Vec<EthercrabSubDevicePreoperational<'maindevice>>,
    subdevice_index: usize,
) -> Result<&'subdevices EthercrabSubDevicePreoperational<'maindevice>, Error> {
    Ok(subdevices.get(subdevice_index).ok_or(anyhow::anyhow!(
        "Index {} out of bounds for subdevices",
        subdevice_index
    ))?)
}
