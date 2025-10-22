use std::sync::Arc;

use control_core::machines::{
    identification::{DeviceHardwareIdentification, DeviceHardwareIdentificationEthercat},
    new::{
        MachineNewParams, get_device_identification_by_role, get_ethercat_device_by_index,
        get_subdevice_by_index,
    },
};
use ethercat_hal::devices::{
    EthercatDevice, SubDeviceIdentityTuple, downcast_device, subdevice_identity_to_tuple,
};
use ethercrab::{SubDevice, SubDeviceRef};
use smol::lock::RwLock;

pub mod aquapath1;

#[cfg(not(feature = "mock-machine"))]
pub mod buffer1;

pub mod extruder1;
pub mod laser;
pub mod mock;
pub mod registry;
pub mod winder2;

pub const VENDOR_QITECH: u16 = 0x0001;
pub const MACHINE_WINDER_V1: u16 = 0x0002;
pub const MACHINE_EXTRUDER_V1: u16 = 0x0004;
pub const MACHINE_LASER_V1: u16 = 0x0006;
pub const MACHINE_MOCK: u16 = 0x0007;
pub const MACHINE_AQUAPATH_V1: u16 = 0x0009;

#[cfg(not(feature = "mock-machine"))]
pub const MACHINE_BUFFER_V1: u16 = 0x0008;

async fn get_device_ident<
    'maindevice,
    'subdevices,
    'device_identifications_identified,
    'ethercat_devices,
    'machine_new_hardware_etehrcat,
    'machine_new_hardware_serial,
    'machine_new_hardware,
>(
    params: &MachineNewParams<
        'maindevice,
        'subdevices,
        'device_identifications_identified,
        'ethercat_devices,
        'machine_new_hardware_etehrcat,
        'machine_new_hardware_serial,
        'machine_new_hardware,
    >,
    role: u16,
) -> Result<DeviceHardwareIdentificationEthercat, anyhow::Error> {
    let device_identification = get_device_identification_by_role(params.device_group, role)?;
    let device_hardware_identification_ethercat =
        match &device_identification.device_hardware_identification {
            DeviceHardwareIdentification::Ethercat(device_hardware_identification_ethercat) => {
                device_hardware_identification_ethercat
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/ExtruderV2::new] Device with role {} is not Ethercat",
                    module_path!(),
                    role
                ));
            }
        };
    return Ok(device_hardware_identification_ethercat.clone());
}

async fn get_ethercat_device<
    'maindevice,
    'subdevices,
    'device_identifications_identified,
    'ethercat_devices,
    'machine_new_hardware_etehrcat,
    'machine_new_hardware_serial,
    'machine_new_hardware,
    T,
>(
    hardware: &&control_core::machines::new::MachineNewHardwareEthercat<
        'maindevice,
        'subdevices,
        'ethercat_devices,
    >,
    params: &MachineNewParams<
        'maindevice,
        'subdevices,
        'device_identifications_identified,
        'ethercat_devices,
        'machine_new_hardware_etehrcat,
        'machine_new_hardware_serial,
        'machine_new_hardware,
    >,
    role: u16,
    expected_identities: Vec<SubDeviceIdentityTuple>,
) -> Result<
    (
        Arc<RwLock<T>>,
        &'subdevices SubDeviceRef<'subdevices, &'subdevices SubDevice>,
    ),
    anyhow::Error,
>
where
    T: 'static + Send + Sync + EthercatDevice,
{
    let device_hardware_identification_ethercat = get_device_ident(params, role).await?;
    let subdevice_index = device_hardware_identification_ethercat.subdevice_index;

    let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
    let subdevice_identity = subdevice.identity();

    let actual_identity = subdevice_identity_to_tuple(&subdevice_identity);

    let mut matched_any_identity = false;
    for identity in expected_identities.clone() {
        if actual_identity == identity {
            matched_any_identity = true;
        }
    }

    if !matched_any_identity {
        return Err(anyhow::anyhow!(
            "[{}::MachineNewTrait/ExtruderV2::new] Device identity mismatch: expected {:?}",
            module_path!(),
            expected_identities
        ));
    }

    let ethercat_device =
        get_ethercat_device_by_index(&hardware.ethercat_devices, subdevice_index)?;
    let device = downcast_device::<T>(ethercat_device).await?;

    {
        let mut device_guard = device.write().await;
        device_guard.set_used(true);
    }

    Ok((device, subdevice))
}
