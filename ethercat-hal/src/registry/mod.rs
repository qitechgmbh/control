use anyhow::Error;
use std::{any::TypeId, collections::HashMap};

use crate::devices::{EthercatDevice, SubDeviceIdentityTuple};

pub mod arc_smol_rwlock;
pub mod r#box;

/// A trait that abstracts over wrapper types for EthercatDevice
/// This allows users to choose between Box<dyn EthercatDevice>, Arc<Mutex<dyn EthercatDevice>>,
/// Arc<RwLock<dyn EthercatDevice>>, etc.
pub trait DeviceWrapper<T: EthercatDevice> {
    /// Create a new wrapped device instance
    fn wrap(device: T) -> Self;
}

/// Type alias for device constructor closure that returns a wrapped device
pub type DeviceNewClosure<W> = Box<dyn Fn() -> Result<W, Error> + Send + Sync>;

/// Trait for registering EtherCAT devices in any registry type
pub trait EthercatDeviceRegistrar {
    /// Register a device type with its identity tuple
    fn register<T>(&mut self, device_identity: SubDeviceIdentityTuple)
    where
        T: EthercatDevice + 'static;

    /// Register a device type with multiple identity tuples
    fn register_multiple<T>(&mut self, device_identities: Vec<SubDeviceIdentityTuple>)
    where
        T: EthercatDevice + 'static,
    {
        for identity in device_identities {
            self.register::<T>(identity);
        }
    }
}

/// Generic EtherCAT device registry that abstracts over the wrapper type
pub struct EthercatDeviceRegistry<W> {
    type_map: HashMap<TypeId, (SubDeviceIdentityTuple, DeviceNewClosure<W>)>,
}

impl<W> EthercatDeviceRegistry<W>
where
    W: 'static,
{
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
        }
    }

    pub fn register<T>(&mut self, device_identity: SubDeviceIdentityTuple)
    where
        T: EthercatDevice + 'static,
        W: DeviceWrapper<T>,
    {
        self.type_map.insert(
            TypeId::of::<T>(),
            (device_identity, Box::new(|| Ok(W::wrap(T::new())))),
        );
    }

    pub fn new_device(
        &self,
        subdevice_identity_tuple: SubDeviceIdentityTuple,
    ) -> Result<W, anyhow::Error> {
        // Find device constructor by identity tuple
        let (_, device_new_closure) = self
            .type_map
            .values()
            .find(|(identity, _)| identity == &subdevice_identity_tuple)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "[{}::EthercatDeviceRegistry::new_device] Device with identity {:?} not found",
                    module_path!(),
                    subdevice_identity_tuple
                )
            })?;

        // Call device constructor
        (device_new_closure)()
    }
}
