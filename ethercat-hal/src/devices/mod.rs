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

use super::devices::{el1008::EL1008, el3204::EL3204};
use crate::{
    devices::el2521::EL2521,
    types::{EthercrabSubDeviceGroupPreoperational, EthercrabSubDevicePreoperational},
};
use anyhow::anyhow;
use bitvec::{order::Lsb0, slice::BitSlice};
use ek1100::EK1100;
use el2002::EL2002;
use el2008::EL2008;
use el2024::EL2024;
use el2522::EL2522;
use el2634::EL2634;
use el2809::EL2809;
use ethercrab::{MainDevice, SubDeviceIdentity};
use std::{any::Any, fmt::Debug, sync::Arc};
use tokio::sync::RwLock;

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

pub trait NewDevice {
    /// Create a new device
    fn new() -> Self
    where
        Self: Sized;
}

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

pub fn device_from_subdevice(
    subdevice_name: &str,
) -> Result<Arc<RwLock<dyn Device>>, anyhow::Error> {
    match subdevice_name {
        "EK1100" => Ok(Arc::new(RwLock::new(EK1100::new()))),
        "EL1008" => Ok(Arc::new(RwLock::new(EL1008::new()))),
        "EL2002" => Ok(Arc::new(RwLock::new(EL2002::new()))),
        "EL2008" => Ok(Arc::new(RwLock::new(EL2008::new()))),
        "EL2024" => Ok(Arc::new(RwLock::new(EL2024::new()))),
        "EL2521" => Ok(Arc::new(RwLock::new(EL2521::new()))),
        "EL2521-0024" => Ok(Arc::new(RwLock::new(EL2521::new()))),
        "EL2522" => Ok(Arc::new(RwLock::new(EL2522::new()))),
        "EL2634" => Ok(Arc::new(RwLock::new(EL2634::new()))),
        "EL2809" => Ok(Arc::new(RwLock::new(EL2809::new()))),
        "EL3001" => Ok(Arc::new(RwLock::new(el3001::EL3001::new()))),
        // "EL4008" => Ok(Arc::new(RwLock::new(EL4008::new()))),
        "EL3204" => Ok(Arc::new(RwLock::new(EL3204::new()))),
        _ => Err(anyhow::anyhow!(
            "[{}::device_from_subdevice] No Driver: {}",
            module_path!(),
            subdevice_name
        )),
    }
}

pub fn devices_from_subdevices<'maindevice, const MAX_SUBDEVICES: usize, const PDI_LEN: usize>(
    group: &mut EthercrabSubDeviceGroupPreoperational<MAX_SUBDEVICES, PDI_LEN>,
    maindevice: &MainDevice,
) -> Result<Vec<Arc<RwLock<dyn Device>>>, anyhow::Error> {
    group
        .iter(maindevice)
        .map(|subdevice| device_from_subdevice(subdevice.name()))
        .collect::<Result<Vec<_>, anyhow::Error>>()
}

/// gets a device by index and transmutes it to the desired type
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

pub async fn specifc_device_from_subdevice<'maindevice, DEVICE: Device>(
    subdevice: &EthercrabSubDevicePreoperational<'maindevice>,
) -> Result<Arc<RwLock<DEVICE>>, anyhow::Error> {
    downcast_device::<DEVICE>(device_from_subdevice(&subdevice.name())?).await
}

pub type SubDeviceIdentityTuple = (u32, u32, u32);
/// function that converts SubDeviceIdentity to tuple
pub fn subdevice_identity_to_tuple(identity: &SubDeviceIdentity) -> SubDeviceIdentityTuple {
    (identity.vendor_id, identity.product_id, identity.revision)
}
