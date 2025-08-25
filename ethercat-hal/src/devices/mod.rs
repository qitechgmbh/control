pub mod ek1100;
pub mod el1002;
pub mod el1008;
pub mod el2002;
pub mod el2004;
pub mod el2008;
pub mod el2024;
pub mod el2521;
pub mod el2522;
pub mod el2634;
pub mod el2809;
pub mod el3001;
pub mod el3021;
pub mod el3024;
pub mod el3062_0030;
pub mod el3204;
pub mod el6021;
pub mod el7031;
pub mod el7031_0030;
pub mod el7041_0052;
// pub mod el4008;
use crate::{
    devices::{
        ek1100::{EK1100, EK1100_IDENTITY_A},
        el1002::{EL1002, EL1002_IDENTITY_A},
        el1008::{EL1008, EL1008_IDENTITY_A},
        el2002::{EL2002, EL2002_IDENTITY_A, EL2002_IDENTITY_B},
        el2004::{EL2004, EL2004_IDENTITY_A},
        el2008::{EL2008, EL2008_IDENTITY_A},
        el2521::{EL2521, EL2521_IDENTITY_0000_A, EL2521_IDENTITY_0000_B, EL2521_IDENTITY_0024_A},
        el2522::{EL2522, EL2522_IDENTITY_A},
        el3001::{EL3001, EL3001_IDENTITY_A},
        el3021::{EL3021, EL3021_IDENTITY_A},
        el3024::{EL3024, EL3024_IDENTITY_A},
        el3062_0030::{EL3062_0030, EL3062_0030_IDENTITY_A},
        el3204::{EL3204, EL3204_IDENTITY_A, EL3204_IDENTITY_B},
        el6021::{
            EL6021, EL6021_IDENTITY_A, EL6021_IDENTITY_B, EL6021_IDENTITY_C, EL6021_IDENTITY_D,
        },
        el7031::{EL7031, EL7031_IDENTITY_A, EL7031_IDENTITY_B},
        el7031_0030::{EL7031_0030, EL7031_0030_IDENTITY_A},
        el7041_0052::{EL7041_0052, EL7041_0052_IDENTITY_A},
    },
    helpers::ethercrab_types::EthercrabSubDeviceGroupPreoperational,
    registry::{
        EthercatDeviceRegistrar,
        arc_smol_rwlock::{ArcSmolRwlockDeviceRegistry, create_default_registry},
    },
};
use anyhow::anyhow;
use bitvec::{order::Lsb0, slice::BitSlice};
use ethercrab::{MainDevice, SubDeviceIdentity};
use once_cell::sync::Lazy;
use smol::lock::RwLock;
use std::{any::Any, fmt::Debug, sync::Arc};

/// A trait for all devices
///
/// provides interface to read and write the PDO data
pub trait EthercatDevice
where
    Self: NewEthercatDevice
        + EthercatDeviceProcessing
        + EthercatDeviceUsed
        + Any
        + Send
        + Sync
        + Debug,
{
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

/// A trait for devices that want to process input and output data
pub trait EthercatDeviceProcessing {
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

/// A trait to ensure a divice can ony be used once
pub trait EthercatDeviceUsed {
    /// Returns true if the device is used
    fn is_used(&self) -> bool;

    /// Sets the device as used
    fn set_used(&mut self, used: bool);
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

/// Internal implementation for populating any registry
pub fn register_default_devices<R: EthercatDeviceRegistrar>(registry: &mut R) {
    // Register all known devices
    registry.register::<EK1100>(EK1100_IDENTITY_A);
    registry.register::<EL1002>(EL1002_IDENTITY_A);
    registry.register::<EL1008>(EL1008_IDENTITY_A);
    registry.register_multiple::<EL2002>(vec![EL2002_IDENTITY_A, EL2002_IDENTITY_B]);
    registry.register::<EL2004>(EL2004_IDENTITY_A);
    registry.register::<EL2008>(EL2008_IDENTITY_A);
    // TODO: implement EL2024 identity
    registry.register_multiple::<EL2521>(vec![
        EL2521_IDENTITY_0000_A,
        EL2521_IDENTITY_0000_B,
        EL2521_IDENTITY_0024_A,
    ]);
    registry.register::<EL2522>(EL2522_IDENTITY_A);
    // TODO: implement EL2634 identity
    // TODO: implement EL2809 identity
    registry.register::<EL3001>(EL3001_IDENTITY_A);
    registry.register::<EL3021>(EL3021_IDENTITY_A);
    registry.register::<EL3024>(EL3024_IDENTITY_A);
    registry.register::<EL3062_0030>(EL3062_0030_IDENTITY_A);
    registry.register_multiple::<EL3204>(vec![EL3204_IDENTITY_A, EL3204_IDENTITY_B]);
    registry.register_multiple::<EL6021>(vec![
        EL6021_IDENTITY_A,
        EL6021_IDENTITY_B,
        EL6021_IDENTITY_C,
        EL6021_IDENTITY_D,
    ]);
    registry.register_multiple::<EL7031>(vec![EL7031_IDENTITY_A, EL7031_IDENTITY_B]);
    registry.register::<EL7031_0030>(EL7031_0030_IDENTITY_A);
    registry.register::<EL7041_0052>(EL7041_0052_IDENTITY_A);
}

/// Static registry instance for efficient device creation
static DEFAULT_DEVICE_REGISTRY: Lazy<ArcSmolRwlockDeviceRegistry> =
    Lazy::new(|| create_default_registry());

/// Construct a device from a subdevice name using the default registry
pub fn device_from_subdevice_identity_tuple(
    subdevice_identity_tuple: SubDeviceIdentityTuple,
) -> Result<Arc<RwLock<dyn EthercatDevice>>, anyhow::Error> {
    DEFAULT_DEVICE_REGISTRY.new_device(subdevice_identity_tuple)
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
