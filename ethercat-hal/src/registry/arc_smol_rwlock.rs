use std::sync::Arc;

use smol::lock::RwLock;

use crate::{
    devices::{EthercatDevice, SubDeviceIdentityTuple},
    registry::{DeviceWrapper, EthercatDeviceRegistrar, EthercatDeviceRegistry},
};

pub type ArcSmolRwlockDeviceRegistry = EthercatDeviceRegistry<Arc<RwLock<dyn EthercatDevice>>>;

/// Implementation for Box wrapper
impl<T: EthercatDevice> DeviceWrapper<T> for Arc<RwLock<dyn EthercatDevice>> {
    fn wrap(device: T) -> Self {
        Arc::new(RwLock::new(device))
    }
}

/// Implement the registrar trait for BoxedDeviceRegistry
impl EthercatDeviceRegistrar for ArcSmolRwlockDeviceRegistry {
    fn register<T>(&mut self, device_identity: SubDeviceIdentityTuple)
    where
        T: EthercatDevice + 'static,
    {
        self.register::<T>(device_identity);
    }
}

/// Create a default ArcSmolRwlockDeviceRegistry with all known EtherCAT devices
pub fn create_default_registry() -> ArcSmolRwlockDeviceRegistry {
    let mut registry = ArcSmolRwlockDeviceRegistry::new();
    register_default_devices(&mut registry);
    registry
}

/// Populate a BoxedDeviceRegistry with all known EtherCAT devices
pub fn register_default_devices(registry: &mut ArcSmolRwlockDeviceRegistry) {
    crate::devices::register_default_devices(registry);
}
