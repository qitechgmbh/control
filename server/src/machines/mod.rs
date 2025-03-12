pub mod winder1;

use crate::ethercat::device_identification::{MachineDeviceIdentification, MachineIdentification};
use anyhow::Error;
use ethercat_hal::{actors::Actor, devices::Device, types::EthercrabSubDevicePreoperational};
use ethercrab::{SubDevice, SubDeviceRef};
use std::sync::Arc;
use tokio::sync::RwLock;
use winder1::WinderV1;

pub enum Machines
where
    Self: Actor + MachineNew,
{
    WinderV1(WinderV1),
}

pub trait MachineNew {
    fn new<'maindevice, 'subdevices>(
        identified_device_group: &Vec<MachineDeviceIdentification>,
        subdevices: &'subdevices Vec<SubDeviceRef<'maindevice, &SubDevice>>,
        devices: &Vec<Arc<RwLock<dyn Device>>>,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}

impl MachineNew for Machines {
    fn new<'maindevice, 'subdevices>(
        identified_device_group: &Vec<MachineDeviceIdentification>,
        subdevices: &'subdevices Vec<SubDeviceRef<'maindevice, &SubDevice>>,
        devices: &Vec<Arc<RwLock<dyn Device>>>,
    ) -> Result<Machines, Error> {
        let machine_identification = &identified_device_group
            .first()
            .unwrap()
            .machine_identification;
        match machine_identification {
            MachineIdentification {
                vendor: VENDOR_QITECH,
                machine: MACHINE_WINDER_V1,
                ..
            } => Ok(Machines::WinderV1(WinderV1::new(
                identified_device_group,
                &subdevices,
                devices,
            )?)),
            _ => Err(anyhow::anyhow!(
                "[{}::Machines::new] Unknown machine",
                module_path!()
            )),
        }
    }
}

impl Actor for Machines {
    fn act(
        &mut self,
        now_ts: u64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        match self {
            Machines::WinderV1(winder) => winder.act(now_ts),
        }
    }
}

const VENDOR_QITECH: u32 = 0x1;
const MACHINE_WINDER_V1: u32 = 0x1;

// validates that all devices in the group have the same machine identification
fn validate_same_machine_identification(
    identified_device_group: &Vec<MachineDeviceIdentification>,
) -> Result<(), Error> {
    let machine_identification = &identified_device_group
        .first()
        .unwrap()
        .machine_identification;
    for device in identified_device_group.iter() {
        if device.machine_identification != *machine_identification {
            return Err(anyhow::anyhow!(
                "[{}::validate_same_machine_identification] Different machine identifications",
                module_path!()
            ));
        }
    }
    Ok(())
}

/// validates that every role is unique
fn validate_no_role_dublicates(
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
fn get_mdi_by_role(
    identified_device_group: &Vec<MachineDeviceIdentification>,
    role: u32,
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
