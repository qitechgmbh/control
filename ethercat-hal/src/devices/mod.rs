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
pub mod el3204;
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
use ethercrab::{MainDevice, SubDeviceIdentity};
use std::{any::Any, fmt::Debug, sync::Arc};
use tokio::sync::RwLock;

/// A trait for all devices
///
/// provides interface to read and write the PDO data
pub trait Device: NewDevice + Any + Send + Sync + Debug {
    /// Input data from the last cycle
    /// `ts` is the timestamp when the input data was sent by the device
    fn input(&mut self, _input: &BitSlice<u8, Lsb0>);

    /// The accepted length of the input data
    fn input_len(&self) -> usize;

    /// automatically validate input length, then calls input
    fn input_checked(&mut self, input: &BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error> {
        // validate input has correct length
        let input_len = self.input_len();
        if input.len() != input_len {
            return Err(anyhow::anyhow!(
                "[{}::Device::input_checked] Input length is {} and must be {} bits",
                module_path!(),
                input.len(),
                input_len
            ));
        }

        self.input(input);

        Ok(())
    }

    /// Output data for the next cycle
    /// `ts` is the timestamp when the output data is predicted to be received by the device
    fn output(&self, _output: &mut BitSlice<u8, Lsb0>);

    /// The accepted length of the output data
    fn output_len(&self) -> usize;

    fn output_checked(&self, output: &mut BitSlice<u8, Lsb0>) -> Result<(), anyhow::Error> {
        self.output(output);

        // validate input has correct length
        let output_len = self.output_len();
        if output.len() != output_len {
            return Err(anyhow::anyhow!(
                "[{}::Device::output_checked] Output length is {} and must be {} bits",
                module_path!(),
                output.len(),
                output_len
            ));
        }

        Ok(())
    }

    /// Write timestamps for current cycle
    fn ts(&mut self, _input_ts: u64, _output_ts: u64);

    fn as_any(&self) -> &dyn Any;
}

/// A constructor trait for devices
///
/// The [`NewDevice::new`] function cannot have params because of it's usage in [`device_from_subdevice`]
pub trait NewDevice {
    /// Create a new device
    fn new() -> Self
    where
        Self: Sized;
}

/// Casts a `dyn Device` to a specific device type
pub async fn downcast_device<T: Device>(
    device: Arc<RwLock<dyn Device>>,
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
pub fn device_from_subdevice(
    subdevice_identity_tuple: SubDeviceIdentityTuple,
) -> Result<Arc<RwLock<dyn Device>>, anyhow::Error> {
    match subdevice_identity_tuple {
        EK1100_IDENTITY_A => Ok(Arc::new(RwLock::new(EK1100::new()))),
        EL1008_IDENTITY_A => Ok(Arc::new(RwLock::new(EL1008::new()))),
        EL2002_IDENTITY_A => Ok(Arc::new(RwLock::new(EL2002::new()))),
        EL2008_IDENTITY_A => Ok(Arc::new(RwLock::new(EL2008::new()))),
        // TODO: implement EL2024 identity
        // EL2024 => Ok(Arc::new(RwLock::new(EL2024::new()))),
        EL2521_IDENTITY_0000_A | EL2521_IDENTITY_0000_B | EL2521_IDENTITY_0024_A => Ok(Arc::new(RwLock::new(EL2521::new()))),
        EL2522_IDENTITY_A => Ok(Arc::new(RwLock::new(EL2522::new()))),
        // TODO: implement EL2634 identity
        // "EL2634" => Ok(Arc::new(RwLock::new(EL2634::new()))),
        // TODO: implement EL2809 identity
        // "EL2809" => Ok(Arc::new(RwLock::new(EL2809::new()))),
        EL3001_IDENTITY_A => Ok(Arc::new(RwLock::new(el3001::EL3001::new()))),
        // "EL4008" => Ok(Arc::new(RwLock::new(EL4008::new()))),
        // TODO: implement EL3204 identity
        // "EL3204" => Ok(Arc::new(RwLock::new(EL3204::new()))),
        _ => Err(anyhow::anyhow!(
            "[{}::device_from_subdevice] No Driver: vendor_id: {:?}, product_id: {:?}, revision: {:?}",
            module_path!(),
            {},
            {},
            {},
        )),
    }
}

/// Array equivalent of [`device_from_subdevice`]
pub fn devices_from_subdevices<'maindevice, const MAX_SUBDEVICES: usize, const PDI_LEN: usize>(
    group: &mut EthercrabSubDeviceGroupPreoperational<MAX_SUBDEVICES, PDI_LEN>,
    maindevice: &MainDevice,
) -> Result<Vec<Arc<RwLock<dyn Device>>>, anyhow::Error> {
    group
        .iter(maindevice)
        .map(|subdevice| subdevice.identity())
        .map(|subdevice_identity| subdevice_identity_to_tuple(&subdevice_identity))
        .map(|subdevice_identity_tuple| device_from_subdevice(subdevice_identity_tuple))
        .collect::<Result<Vec<_>, anyhow::Error>>()
}

/// Casts a `dyn Device` from an array into a specific device type using [`downcast_device`]
pub async fn specific_device_from_devices<DEVICE: Device>(
    devices: &Vec<Arc<RwLock<dyn Device>>>,
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
