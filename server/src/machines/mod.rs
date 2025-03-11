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
    Self: Actor,
{
    WinderV1(WinderV1),
}

impl Machines {
    pub fn new<'maindevice, 'subdevices>(
        identified_device_group: &Vec<MachineDeviceIdentification>,
        subdevices: &'subdevices Vec<SubDeviceRef<'maindevice, &SubDevice>>,
        devices: &Vec<Option<Arc<RwLock<dyn Device>>>>,
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
            _ => Err(anyhow::anyhow!("Unknown machine")),
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
            return Err(anyhow::anyhow!("Different machine identifications"));
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
            return Err(anyhow::anyhow!("Role dublicate"));
        }
        roles.push(device.role);
    }
    Ok(())
}

/// get a device with a device group
fn get_device_by_role(
    identified_device_group: &Vec<MachineDeviceIdentification>,
    role: u32,
) -> Result<(usize, &MachineDeviceIdentification), Error> {
    for (i, device) in identified_device_group.iter().enumerate() {
        if device.role == role {
            return Ok((i, device));
        }
    }
    Err(anyhow::anyhow!("Device not found"))
}

pub fn get_device_by_index<'maindevice>(
    devices: &Vec<Option<Arc<RwLock<dyn Device>>>>,
    subdevice_index: usize,
) -> Result<Arc<RwLock<dyn Device>>, Error> {
    Ok(devices
        .get(subdevice_index)
        .ok_or(anyhow::anyhow!(
            "Index {} out of bounds for devices",
            subdevice_index
        ))?
        .clone()
        .ok_or(anyhow::anyhow!(
            "No device driver with index {} is None",
            subdevice_index
        ))?)
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
