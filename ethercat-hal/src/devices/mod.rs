pub mod ek1100;
pub mod el1008;
pub mod el2002;
pub mod el2008;
pub mod el2024;
pub mod el2521;
pub mod el2522;
pub mod el2634;
pub mod el2809;
pub mod el3001;
pub mod el3021;
pub mod el3024;
pub mod el3204;
pub mod el6021;
pub mod el7031;
pub mod el7031_0030;
pub mod el7041_0052;
// pub mod el4008;

use super::devices::el1008::EL1008;
use crate::{devices::el2521::EL2521, types::EthercrabSubDeviceGroupPreoperational};
use anyhow::anyhow;
use bitvec::{order::Lsb0, slice::BitSlice};
use ek1100::{EK1100, EK1100_IDENTITY_A};
use el1008::EL1008_IDENTITY_A;
use el2002::{EL2002, EL2002_IDENTITY_A};
use el2008::{EL2008, EL2008_IDENTITY_A};
use el2521::{EL2521_IDENTITY_0000_A, EL2521_IDENTITY_0000_B, EL2521_IDENTITY_0024_A};
use el2522::{EL2522, EL2522_IDENTITY_A};
use el3001::EL3001_IDENTITY_A;
use el3021::EL3021_IDENTITY_A;
use el3024::EL3024_IDENTITY_A;
use el6021::EL6021_IDENTITY_A;
use el7031::{EL7031_IDENTITY_A, EL7031_IDENTITY_B};
use el7031_0030::EL7031_0030_IDENTITY_A;
use el7041_0052::EL7041_0052_IDENTITY_A;
use ethercrab::{MainDevice, SubDeviceIdentity};
use smol::lock::RwLock;
use std::{any::Any, fmt::Debug, sync::Arc};

/// A trait for all devices
///
/// provides interface to read and write the PDO data
pub trait EthercatDevice: NewEthercatDevice + Any + Send + Sync + Debug {
    /// Input data from the last cycle
    /// `ts` is the timestamp when the input data was sent by the device
    fn input(&mut self, _input: &BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error>;

    /// The accepted length of the input data
    fn input_len(&self) -> usize;

    /// automatically validate input length, then calls input
    fn input_checked(&mut self, input: &BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error> {
        // validate input has correct length
        let expected = self.input_len();
        let actual = input.len();
        if actual != expected {
            return Err(anyhow::anyhow!(
                "[{}::Device::input_checked] Input length is {} ({} bytes) and must be {} bits ({} bytes)",
                module_path!(),
                actual,
                actual / 8,
                expected,
                expected / 8
            ));
        }
        self.input(input)
    }

    /// Devices can override this function if they want to post process the input data
    /// This might be the case if the pdo is not what is needed in the io layer
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// Devices can override this function if they want to pre process the output data
    /// This might be the case if the pdo is not what is needed in the io layer
    fn output_pre_process(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// Output data for the next cycle
    /// `ts` is the timestamp when the output data is predicted to be received by the device
    fn output(&self, _output: &mut BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error>;

    /// The accepted length of the output data
    fn output_len(&self) -> usize;

    fn output_checked(&self, output: &mut BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error> {
        self.output(output)?;

        // validate input has correct length
        let expected = self.output_len();
        let actual = output.len();
        if output.len() != expected {
            return Err(anyhow::anyhow!(
                "[{}::Device::output_checked] Output length is {} ({} bytes) and must be {} bits ({} bytes)",
                module_path!(),
                actual,
                actual / 8,
                expected,
                expected / 8
            ));
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any;
}

/// A constructor trait for devices
///
/// The [`NewDevice::new`] function cannot have params because of it's usage in [`device_from_subdevice`]
pub trait NewEthercatDevice {
    /// Create a new device
    fn new() -> Self
    where
        Self: Sized;
}

/// Casts a `dyn Device` to a specific device type
pub async fn downcast_device<T: EthercatDevice>(
    device: Arc<RwLock<dyn EthercatDevice>>,
) -> Result<Arc<RwLock<T>>, anyhow::Error> {
    // Acquire a read lock on the RwLock
    let read_lock = device.read().await;

    // Check if the inner type can be downcasted to T
    if read_lock.as_any().is::<T>() {
        // Clone the Arc and return it as the desired type
        let cloned_device = Arc::clone(&device);
        // Transmute the Arc to the desired type
        unsafe {
            Ok(Arc::from_raw(
                Arc::into_raw(cloned_device) as *const RwLock<T>
            ))
        }
    } else {
        Err(anyhow!(
            "[{}::downcast_device] Downcast failed",
            module_path!()
        ))
    }
}

/// Construct a device from a subdevice name
pub fn device_from_subdevice_identity_tuple(
    subdevice_identity_tuple: SubDeviceIdentityTuple,
) -> Result<Arc<RwLock<dyn EthercatDevice>>, anyhow::Error> {
    match subdevice_identity_tuple {
        EK1100_IDENTITY_A => Ok(Arc::new(RwLock::new(EK1100::new()))),
        EL1008_IDENTITY_A => Ok(Arc::new(RwLock::new(EL1008::new()))),
        EL2002_IDENTITY_A => Ok(Arc::new(RwLock::new(EL2002::new()))),
        EL2008_IDENTITY_A => Ok(Arc::new(RwLock::new(EL2008::new()))),
        // TODO: implement EL2024 identity
        // EL2024 => Ok(Arc::new(RwLock::new(EL2024::new()))),
        EL2521_IDENTITY_0000_A | EL2521_IDENTITY_0000_B | EL2521_IDENTITY_0024_A => {
            Ok(Arc::new(RwLock::new(EL2521::new())))
        }
        EL2522_IDENTITY_A => Ok(Arc::new(RwLock::new(EL2522::new()))),
        // TODO: implement EL2634 identity
        // "EL2634" => Ok(Arc::new(RwLock::new(EL2634::new()))),
        // TODO: implement EL2809 identity
        // "EL2809" => Ok(Arc::new(RwLock::new(EL2809::new()))),
        EL3001_IDENTITY_A => Ok(Arc::new(RwLock::new(el3001::EL3001::new()))),
        EL3021_IDENTITY_A => Ok(Arc::new(RwLock::new(el3021::EL3021::new()))),
        EL3024_IDENTITY_A => Ok(Arc::new(RwLock::new(el3024::EL3024::new()))),
        EL6021_IDENTITY_A => Ok(Arc::new(RwLock::new(el6021::EL6021::new()))),
        // "EL4008" => Ok(Arc::new(RwLock::new(EL4008::new()))),
        // TODO: implement EL3204 identity
        // "EL3204" => Ok(Arc::new(RwLock::new(EL3204::new()))),
        EL7031_IDENTITY_A | EL7031_IDENTITY_B => Ok(Arc::new(RwLock::new(el7031::EL7031::new()))),
        EL7031_0030_IDENTITY_A => Ok(Arc::new(RwLock::new(el7031_0030::EL7031_0030::new()))),
        EL7041_0052_IDENTITY_A => Ok(Arc::new(RwLock::new(el7041_0052::EL7041_0052::new()))),
        _ => Err(anyhow::anyhow!(
            "[{}::device_from_subdevice] No Driver: vendor_id: {:?}, product_id: {:?}, revision: {:?}",
            module_path!(),
            subdevice_identity_tuple.0,
            subdevice_identity_tuple.1,
            subdevice_identity_tuple.2,
        )),
    }
}

/// Construct a device from a subdevice
/// Combines [`subdevice_identity_to_tuple`] and [`device_from_subdevice_identity_tuple`]
pub fn device_from_subdevice_identity(
    subdevice: &SubDeviceIdentity,
) -> Result<Arc<RwLock<dyn EthercatDevice>>, anyhow::Error> {
    let subdevice_identity_tuple = subdevice_identity_to_tuple(subdevice);
    device_from_subdevice_identity_tuple(subdevice_identity_tuple)
}

/// Array equivalent of [`device_from_subdevice`]
pub fn devices_from_subdevices<'maindevice, const MAX_SUBDEVICES: usize, const PDI_LEN: usize>(
    group: &mut EthercrabSubDeviceGroupPreoperational<MAX_SUBDEVICES, PDI_LEN>,
    maindevice: &MainDevice,
) -> Result<Vec<Arc<RwLock<dyn EthercatDevice>>>, anyhow::Error> {
    group
        .iter(maindevice)
        .map(|subdevice| subdevice.identity())
        .map(|subdevice_identity| device_from_subdevice_identity(&subdevice_identity))
        .collect::<Result<Vec<_>, anyhow::Error>>()
}

/// Casts a `dyn Device` from an array into a specific device type using [`downcast_device`]
pub async fn specific_device_from_devices<DEVICE: EthercatDevice>(
    devices: &Vec<Arc<RwLock<dyn EthercatDevice>>>,
    index: usize,
) -> Result<Arc<RwLock<DEVICE>>, anyhow::Error> {
    downcast_device::<DEVICE>(devices.get(index).cloned().ok_or({
        anyhow!(
            "[{}::specific_device_from_devices] Couldnt find device with matching type at {}",
            module_path!(),
            index
        )
    })?)
    .await
}

pub type SubDeviceIdentityTuple = (u32, u32, u32);
/// function that converts SubDeviceIdentity to tuple
pub fn subdevice_identity_to_tuple(identity: &SubDeviceIdentity) -> SubDeviceIdentityTuple {
    (identity.vendor_id, identity.product_id, identity.revision)
}
